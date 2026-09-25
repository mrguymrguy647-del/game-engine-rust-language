//! Drawing 3D on the graphics card (GPU) with wgpu.
//!
//! This whole file only exists when the `gpu` feature is on (it is by
//! default). It does the same job as the CPU path in `render3d.rs`, but the
//! graphics card does the heavy work:
//!
//! 1. `Canvas::draw_mesh` doesn't draw anything right away. It records
//!    *which* mesh to draw and *where* (a matrix), grouped by mesh.
//! 2. Each mesh is sent to the GPU only once and then kept there (the
//!    *mesh cache*).
//! 3. When the 3D picture is needed (the canvas is drawn on in 2D again, or
//!    the frame ends), [`GpuRenderer::flush`] draws every recorded object
//!    with one draw call per mesh (*instancing*), onto a copy of the
//!    canvas, and copies the result back into the canvas's pixels.
//!
//! The game loop and the window don't change at all: the canvas's pixels
//! simply contain the 3D picture afterwards, just as with the CPU renderer.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasherDefault, Hash, Hasher};

use bytemuck::{Pod, Zeroable};

use crate::render3d::{Camera3D, Mesh, Transform};

/// One corner of one triangle, laid out exactly as the shader reads it.
///
/// `#[repr(C)]` fixes the order and layout of the fields in memory, and
/// `Pod` ("plain old data") promises it's just bytes, so `bytemuck` can
/// hand a whole `Vec<Vertex>` to the GPU as a byte slice without copying.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [u8; 4],
}

/// Per-object data: the object's transform matrix.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Instance {
    model: [[f32; 4]; 4],
}

/// Settings shared by one batch of drawing (see `Globals` in `gpu.wgsl`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    sun: [f32; 4],
}

/// Where a mesh's two lists live in memory, and how long they are. The same
/// `Mesh` drawn many times in a frame always has the same address.
type Address = (usize, usize, usize, usize);

/// How the cache recognizes a mesh: its address plus a fingerprint of its
/// contents. The fingerprint matters because meshes can change (their
/// fields are public), and because a temporary mesh can reuse the memory of
/// one that was just dropped.
type MeshKey = (Address, u64);

fn address_of(mesh: &Mesh) -> Address {
    (
        mesh.vertices.as_ptr() as usize,
        mesh.vertices.len(),
        mesh.triangles.as_ptr() as usize,
        mesh.triangles.len(),
    )
}

/// A summary of *everything* in the mesh. Two meshes with different contents
/// get different fingerprints (with overwhelming probability).
fn fingerprint(mesh: &Mesh) -> u64 {
    let mut hasher = DefaultHasher::new();
    for &v in &mesh.vertices {
        hash_vertex(v, &mut hasher);
    }
    for &t in &mesh.triangles {
        hash_triangle(t, &mut hasher);
    }
    hasher.finish()
}

/// A few parts of a mesh: its first, middle and last corner and triangle.
/// Much quicker to compare than working out a full fingerprint, and enough
/// to notice that a different mesh now lives at the same address.
#[derive(Clone, Copy, PartialEq)]
struct Sample {
    vertices: [Option<crate::math::Vec3>; 3],
    triangles: [Option<crate::render3d::Triangle>; 3],
}

fn sample(mesh: &Mesh) -> Sample {
    let picks = |len: usize| [0, len / 2, len.saturating_sub(1)];
    Sample {
        vertices: picks(mesh.vertices.len()).map(|i| mesh.vertices.get(i).copied()),
        triangles: picks(mesh.triangles.len()).map(|i| mesh.triangles.get(i).copied()),
    }
}

fn hash_vertex(v: crate::math::Vec3, hasher: &mut DefaultHasher) {
    // Floats can't be hashed directly (NaN isn't equal to itself), so hash their bits.
    [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()].hash(hasher);
}

fn hash_triangle(t: crate::render3d::Triangle, hasher: &mut DefaultHasher) {
    t.corners.hash(hasher);
    [t.color.r, t.color.g, t.color.b].hash(hasher);
}

/// What we learned about a mesh address during the current batch.
struct Seen {
    sample: Sample,
    fingerprint: u64,
}

/// A very fast hash function for the lookup tables below.
///
/// The standard library's default hasher is built to resist deliberate
/// attacks from untrusted data, which makes it slow for small keys like
/// ours (it showed up as most of the time spent per object). This one is
/// the simple design the Rust compiler itself uses ("FxHash"): mix in each
/// number with a rotate, an XOR and a multiply.
#[derive(Default)]
struct FastHasher(u64);

impl Hasher for FastHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u64(byte as u64);
        }
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// A `HashMap` that uses [`FastHasher`].
type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;

/// A mesh that lives on the GPU.
struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    /// The flush during which it was last drawn.
    last_used: u64,
}

/// Meshes not drawn for this many flushes are removed from the GPU.
const EVICT_AFTER_FLUSHES: u64 = 120;

/// Everything needed to draw 3D on the GPU for one canvas size.
pub(crate) struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// "GPU: " plus the graphics card's name and the API used to talk to it.
    description: String,
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    /// The picture being drawn. Same size and byte layout as the canvas.
    color_texture: wgpu::Texture,
    color_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    /// Where the finished picture is copied, so the CPU can read it.
    readback_buffer: wgpu::Buffer,
    /// Rows in `readback_buffer` are padded to a multiple of 256 bytes.
    padded_bytes_per_row: u32,
    width: u32,
    height: u32,
    instance_buffer: wgpu::Buffer,

    meshes: FastMap<MeshKey, GpuMesh>,
    /// Fingerprints worked out during the current batch, so each mesh is
    /// fully fingerprinted only once per batch, however often it's drawn.
    seen: FastMap<Address, Seen>,
    /// The objects recorded since the last flush, grouped by mesh.
    pending: FastMap<MeshKey, Vec<Instance>>,
    /// The camera for the pending objects.
    camera: Option<Camera3D>,
    /// Whether the next flush starts with an empty depth buffer.
    clear_depth: bool,
    flushes: u64,
}

impl GpuRenderer {
    /// Starts up the GPU. This can fail, for example on a computer with no
    /// usable graphics driver; the canvas then falls back to the CPU.
    pub(crate) fn new(width: usize, height: usize) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err(String::from("the canvas has no pixels"));
        }
        let (width, height) = (width as u32, height as u32);

        // 1. An *instance* is the entry point to wgpu. `new_without_display_handle_from_env`
        //    means we won't draw into a window (just into a texture), and lets
        //    the WGPU_BACKEND environment variable pick Vulkan, Metal, DX12 or GL.
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        // 2. An *adapter* is one graphics card (or a software imitation of one).
        //    wgpu's setup functions are `async`; `pollster::block_on` simply
        //    waits for them to finish.
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .map_err(|err| format!("no graphics adapter found: {err}"))?;
        let info = adapter.get_info();
        let description = format!("GPU: {} ({:?})", info.name, info.backend);

        // 3. A *device* is our connection to the adapter; the *queue* sends it work.
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("duckforge"),
            ..Default::default()
        }))
        .map_err(|err| format!("could not open the graphics device: {err}"))?;

        // 4. The shader programs, and a *pipeline*: the full recipe for drawing.
        let shader = device.create_shader_module(wgpu::include_wgsl!("gpu.wgsl"));

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh"),
            bind_group_layouts: &[Some(&globals_layout)],
            immediate_size: 0,
        });

        let vertex_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Unorm8x4];
        let instance_attributes = wgpu::vertex_attr_array![
            2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4
        ];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    // Buffer 0: the mesh's corners, one per shader run.
                    Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attributes,
                    }),
                    // Buffer 1: one matrix per *object*. `Instance` step mode means
                    // the GPU moves to the next matrix only for the next copy
                    // of the whole mesh. That's instancing.
                    Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attributes,
                    }),
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // Our rule: corners go clockwise when seen from the front.
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back), // back-face culling, in hardware
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                // Depth is `near / distance`, so bigger means closer (like the CPU's 1 / z).
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: COLOR_FORMAT,
                    blend: None, // just replace the pixel
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        // 5. The picture, the depth buffer, and a buffer to copy the picture into.
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        // Copies from a texture into a buffer need each row to take a multiple of 256 bytes.
        let padded_bytes_per_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: padded_bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let instance_buffer = create_instance_buffer(&device, 1024);

        Ok(Self {
            color_view: color_texture.create_view(&Default::default()),
            depth_view: depth_texture.create_view(&Default::default()),
            color_texture,
            device,
            queue,
            description,
            pipeline,
            globals_buffer,
            globals_bind_group,
            readback_buffer,
            padded_bytes_per_row,
            width,
            height,
            instance_buffer,
            meshes: FastMap::default(),
            seen: FastMap::default(),
            pending: FastMap::default(),
            camera: None,
            clear_depth: true,
            flushes: 0,
        })
    }

    /// "GPU: " plus the graphics card's name and which API wgpu uses to talk to it.
    pub(crate) fn description(&self) -> &str {
        &self.description
    }

    /// Called by `Canvas::clear`: forget pending drawing and start the next
    /// batch with an empty depth buffer.
    pub(crate) fn clear(&mut self) {
        self.pending.clear();
        self.seen.clear();
        self.camera = None;
        self.clear_depth = true;
    }

    /// Records one object to draw at the next flush.
    pub(crate) fn draw(
        &mut self,
        mesh: &Mesh,
        transform: &Transform,
        camera: &Camera3D,
        pixels: &mut [u32],
    ) {
        // Everything in one batch shares a camera. A new camera? Draw what we have first.
        if self.camera.is_some_and(|current| current != *camera) {
            self.flush(pixels);
        }
        self.camera = Some(*camera);

        let key = self.key_for(mesh);
        if let Some(cached) = self.meshes.get_mut(&key) {
            cached.last_used = self.flushes;
        } else {
            self.upload(key, mesh);
        }
        let instance = Instance {
            model: transform.matrix().columns,
        };
        self.pending.entry(key).or_default().push(instance);
    }

    /// Works out the cache key for `mesh`, fingerprinting it only if this
    /// address hasn't been seen this batch, or seems to hold something else now.
    fn key_for(&mut self, mesh: &Mesh) -> MeshKey {
        let address = address_of(mesh);
        let quick = sample(mesh);
        match self.seen.get(&address) {
            Some(seen) if seen.sample == quick => (address, seen.fingerprint),
            _ => {
                let print = fingerprint(mesh);
                let seen = Seen {
                    sample: quick,
                    fingerprint: print,
                };
                self.seen.insert(address, seen);
                (address, print)
            }
        }
    }

    /// Sends a copy of `mesh` to the GPU.
    fn upload(&mut self, key: MeshKey, mesh: &Mesh) {
        // One GPU vertex per triangle corner, each carrying the triangle's color.
        let vertices: Vec<Vertex> = mesh
            .triangles
            .iter()
            .flat_map(|triangle| {
                let color = [triangle.color.r, triangle.color.g, triangle.color.b, 255];
                triangle.corners.map(|i| {
                    let p = mesh.vertices[i];
                    Vertex {
                        position: [p.x, p.y, p.z],
                        color,
                    }
                })
            })
            .collect();

        use wgpu::util::DeviceExt; // brings `create_buffer_init` into scope
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.meshes.insert(
            key,
            GpuMesh {
                vertex_buffer,
                vertex_count: vertices.len() as u32,
                last_used: self.flushes,
            },
        );
    }

    /// Draws everything recorded since the last flush on top of `pixels`
    /// (the canvas), then copies the result back into `pixels`.
    pub(crate) fn flush(&mut self, pixels: &mut [u32]) {
        let Some(camera) = self.camera else {
            return;
        };
        if self.pending.is_empty() {
            return;
        }

        // --- Send this batch's data to the GPU. ---

        // The canvas as it is now, so the 3D is drawn over any 2D background.
        // A canvas pixel `0x00RRGGBB` is stored as the bytes B, G, R, 0: exactly
        // the `Bgra8Unorm` texture format, so no conversion is needed.
        self.queue.write_texture(
            self.color_texture.as_image_copy(),
            bytemuck::cast_slice(pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: Some(self.height),
            },
            self.color_texture.size(),
        );

        let aspect = self.width as f32 / self.height as f32;
        let view_projection = camera.projection_matrix(aspect) * camera.view_matrix();
        let sun = crate::render3d::sun_direction();
        let globals = Globals {
            view_projection: view_projection.columns,
            camera_position: [camera.position.x, camera.position.y, camera.position.z, 0.0],
            sun: [sun.x, sun.y, sun.z, crate::render3d::AMBIENT_LIGHT],
        };
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        // All objects' matrices go into one buffer, mesh after mesh.
        let batches: Vec<(MeshKey, Vec<Instance>)> = self.pending.drain().collect();
        let total: usize = batches.iter().map(|(_, instances)| instances.len()).sum();
        let needed = (total * std::mem::size_of::<Instance>()) as u64;
        if self.instance_buffer.size() < needed {
            let capacity = total.next_power_of_two();
            self.instance_buffer = create_instance_buffer(&self.device, capacity);
        }
        let all_instances: Vec<Instance> = batches
            .iter()
            .flat_map(|(_, list)| list.iter().copied())
            .collect();
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&all_instances),
        );

        // --- Record the GPU's work: one render pass, then a copy. ---

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let depth_load = if self.clear_depth {
                wgpu::LoadOp::Clear(0.0) // 0.0 = infinitely far away
            } else {
                wgpu::LoadOp::Load // keep depths from earlier batches this frame
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("meshes"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // start from the uploaded canvas
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: depth_load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);

            // One draw call per mesh, however many copies of it there are.
            let mut first = 0u32;
            for (key, instances) in &batches {
                let count = instances.len() as u32;
                let mesh = &self.meshes[key];
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                pass.draw(0..mesh.vertex_count, first..first + count);
                first += count;
            }
        }
        encoder.copy_texture_to_buffer(
            self.color_texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            self.color_texture.size(),
        );
        self.queue.submit([encoder.finish()]);

        // --- Wait for the GPU, then copy the picture back into the canvas. ---

        self.readback_buffer
            .map_async(wgpu::MapMode::Read, .., |result| {
                result.expect("could not read the picture back from the GPU");
            });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the GPU stopped responding");
        {
            let bytes = self
                .readback_buffer
                .get_mapped_range(..)
                .expect("the picture should be readable after mapping");
            let row_bytes = self.width as usize * 4;
            for (y, row) in pixels.chunks_exact_mut(self.width as usize).enumerate() {
                let start = y * self.padded_bytes_per_row as usize;
                let source = &bytes[start..start + row_bytes];
                for (pixel, bgra) in row.iter_mut().zip(source.chunks_exact(4)) {
                    // Bytes B, G, R, A back into 0x00RRGGBB (dropping A).
                    *pixel = u32::from_le_bytes([bgra[0], bgra[1], bgra[2], 0]);
                }
            }
        }
        self.readback_buffer.unmap();

        // Forget meshes that haven't been drawn for a while, to free GPU memory.
        self.flushes += 1;
        let now = self.flushes;
        self.meshes
            .retain(|_, mesh| now - mesh.last_used <= EVICT_AFTER_FLUSHES);
        self.seen.clear(); // next batch: check every mesh again
        self.clear_depth = false;
    }
}

/// The picture's format on the GPU. Blue, green, red, alpha: the same byte
/// order as the canvas's pixels.
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("instances"),
        size: (capacity.max(1) * std::mem::size_of::<Instance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;
    use crate::color::Color;
    use crate::math::{Rect, Vec3, vec3};
    use crate::render3d::{Camera3D, Mesh, Renderer, Transform};

    /// A canvas that draws 3D on the GPU, or `None` (and the test is
    /// skipped) on a machine without a usable graphics adapter.
    fn gpu_canvas(width: usize, height: usize) -> Option<Canvas> {
        let mut canvas = Canvas::new(width, height);
        canvas.set_renderer(Renderer::Gpu);
        // Draw one mesh off-screen to start the GPU up.
        let camera = Camera3D::new(Vec3::ZERO);
        canvas.draw_mesh(
            &Mesh::cube(Color::RED),
            &Transform::at(vec3(0.0, 0.0, -5.0)),
            &camera,
        );
        canvas.clear(Color::BLACK);
        if canvas.renderer_name().starts_with("GPU:") {
            Some(canvas)
        } else {
            eprintln!("no GPU adapter: skipping this test");
            None
        }
    }

    /// 2D background, a floor, several shapes at odd angles, and 2D on top.
    fn draw_scene(canvas: &mut Canvas) {
        let camera = Camera3D::looking_at(vec3(-3.0, 4.0, -7.0), vec3(0.0, 0.5, 0.0));
        canvas.clear(Color::from_hex(0x20_30_40));
        canvas.fill_rect(Rect::new(0.0, 0.0, 40.0, 30.0), Color::GREEN); // under the 3D
        let floor = Mesh::checkerboard(8, 1.0, Color::GRAY, Color::WHITE);
        canvas.draw_mesh(&floor, &Transform::default(), &camera);
        let shapes = [
            (Mesh::cube(Color::RED), vec3(-1.5, 0.5, 0.0)),
            (Mesh::sphere(Color::BLUE, 16), vec3(1.2, 0.7, 0.5)),
            (Mesh::pyramid(Color::YELLOW), vec3(0.0, 0.5, 2.0)),
        ];
        for (i, (mesh, position)) in shapes.iter().enumerate() {
            let transform = Transform::at(*position)
                .rotated(vec3(0.3 * i as f32, 0.8 + i as f32, 0.1))
                .scaled(vec3(1.0, 1.3, 0.9));
            canvas.draw_mesh(mesh, &transform, &camera);
        }
        canvas.fill_rect(Rect::new(120.0, 90.0, 40.0, 30.0), Color::PURPLE); // over the 3D
    }

    #[test]
    fn gpu_picture_matches_the_cpu_picture() {
        let Some(mut gpu) = gpu_canvas(160, 120) else {
            return;
        };
        let mut cpu = Canvas::new(160, 120);
        draw_scene(&mut gpu);
        draw_scene(&mut cpu);
        gpu.flush_3d();

        // The two renderers may round a few edge pixels differently, and
        // their colors may differ by a tiny bit. Everything else must match.
        let mut different = 0;
        for (&g, &c) in gpu.pixels().iter().zip(cpu.pixels()) {
            let (g, c) = (Color::from_hex(g), Color::from_hex(c));
            let gap = (g.r.abs_diff(c.r))
                .max(g.g.abs_diff(c.g))
                .max(g.b.abs_diff(c.b));
            if gap > 3 {
                different += 1;
            }
        }
        let share = different as f32 / gpu.pixels().len() as f32;
        assert!(share < 0.02, "{:.1}% of pixels differ", share * 100.0);
    }

    #[test]
    fn two_dimensional_drawing_layers_under_and_over_3d() {
        let Some(mut canvas) = gpu_canvas(64, 48) else {
            return;
        };
        let camera = Camera3D::new(vec3(0.0, 0.0, -3.0));
        canvas.clear(Color::BLACK);
        canvas.fill_rect(Rect::new(0.0, 0.0, 64.0, 48.0), Color::GREEN); // background
        canvas.draw_mesh(&Mesh::cube(Color::RED), &Transform::default(), &camera);
        canvas.fill_rect(Rect::new(30.0, 22.0, 4.0, 4.0), Color::WHITE); // on top
        canvas.flush_3d();

        assert_eq!(canvas.get_pixel(1, 1), Some(Color::GREEN)); // background shows
        assert_eq!(canvas.get_pixel(32, 24), Some(Color::WHITE)); // 2D over 3D
        let cube = canvas.get_pixel(26, 24).unwrap();
        // Red, shaded to 66% brightness: about (153, 40, 40).
        assert!(
            cube.r > 100 && cube.r > 3 * cube.g,
            "the cube is drawn: {cube:?}"
        );
    }

    #[test]
    fn temporary_meshes_keep_their_own_colors() {
        // Meshes made on the spot are dropped right after drawing, so the
        // next one often lands at the same memory address.
        let Some(mut canvas) = gpu_canvas(64, 48) else {
            return;
        };
        let camera = Camera3D::new(vec3(0.0, 0.0, -4.0));
        canvas.clear(Color::BLACK);
        canvas.draw_mesh(
            &Mesh::cube(Color::RED),
            &Transform::at(vec3(-1.0, 0.0, 0.0)),
            &camera,
        );
        canvas.draw_mesh(
            &Mesh::cube(Color::BLUE),
            &Transform::at(vec3(1.0, 0.0, 0.0)),
            &camera,
        );
        canvas.flush_3d();

        let left = canvas.get_pixel(20, 24).unwrap();
        let right = canvas.get_pixel(44, 24).unwrap();
        assert!(left.r > left.b, "left cube should be red: {left:?}");
        assert!(right.b > right.r, "right cube should be blue: {right:?}");
    }

    #[test]
    fn edited_meshes_are_uploaded_again() {
        let Some(mut canvas) = gpu_canvas(32, 24) else {
            return;
        };
        let camera = Camera3D::new(vec3(0.0, 0.0, -2.0));
        let mut mesh = Mesh::cube(Color::RED);
        canvas.draw_mesh(&mesh, &Transform::default(), &camera);
        canvas.flush_3d();
        assert!(canvas.get_pixel(16, 12).unwrap().r > 100);

        mesh = mesh.with_color(Color::BLUE); // same sizes, likely the same memory
        canvas.clear(Color::BLACK);
        canvas.draw_mesh(&mesh, &Transform::default(), &camera);
        canvas.flush_3d();
        let pixel = canvas.get_pixel(16, 12).unwrap();
        // Blue, shaded to 66% brightness: about (46, 86, 152).
        assert!(
            pixel.b > 100 && pixel.b > 3 * pixel.r,
            "should be blue now: {pixel:?}"
        );
    }

    #[test]
    fn thousands_of_instances() {
        let Some(mut canvas) = gpu_canvas(100, 100) else {
            return;
        };
        let camera = Camera3D::looking_at(vec3(0.0, 60.0, -1.0), Vec3::ZERO);
        let cube = Mesh::cube(Color::ORANGE);
        canvas.clear(Color::BLACK);
        for x in -50..50 {
            for z in -50..50 {
                let at = vec3(x as f32 * 0.6, 0.0, z as f32 * 0.6);
                canvas.draw_mesh(&cube, &Transform::at(at).sized(0.4), &camera);
            }
        }
        canvas.flush_3d();
        let lit = canvas.pixels().iter().filter(|&&p| p != 0).count();
        assert!(
            lit > 3_000,
            "10,000 cubes should cover much of the view ({lit} pixels lit)"
        );
    }
}
