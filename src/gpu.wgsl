// The programs that run on the graphics card, written in WGSL (the WebGPU
// Shading Language). The GPU runs `vs_main` once for every corner of every
// triangle, then `fs_main` once for every pixel the triangle covers.

// Settings shared by everything drawn in one batch.
struct Globals {
    // World space to the screen: camera.projection_matrix() * camera.view_matrix().
    view_projection: mat4x4<f32>,
    // Where the camera is (w is unused).
    camera_position: vec4<f32>,
    // xyz: the direction sunlight comes from. w: the ambient light level.
    sun: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexIn {
    // From the mesh: one corner of one triangle.
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    // From the object being drawn: its transform matrix, one column at a time.
    @location(2) model_0: vec4<f32>,
    @location(3) model_1: vec4<f32>,
    @location(4) model_2: vec4<f32>,
    @location(5) model_3: vec4<f32>,
};

struct VertexOut {
    // Where the corner lands on screen. The GPU divides x, y and z by w.
    @builtin(position) clip_position: vec4<f32>,
    // Passed on to fs_main, blended across the triangle.
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    let model = mat4x4<f32>(v.model_0, v.model_1, v.model_2, v.model_3);
    let world = model * vec4<f32>(v.position, 1.0);

    var out: VertexOut;
    out.clip_position = globals.view_projection * world;
    out.world_position = world.xyz;
    out.color = v.color.rgb;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // The triangle's normal. `dpdx` and `dpdy` say how the world position
    // changes from this pixel to its right and lower neighbors: two
    // directions lying flat in the triangle. Their cross product sticks
    // straight out of it, just like `(b - a).cross(c - a)` on the CPU.
    var normal = normalize(cross(dpdx(in.world_position), dpdy(in.world_position)));
    // Make sure it points out of the front, towards the camera.
    if dot(normal, globals.camera_position.xyz - in.world_position) < 0.0 {
        normal = -normal;
    }

    // The same flat shading as the CPU renderer.
    let sunlight = max(dot(normal, globals.sun.xyz), 0.0);
    let ambient = globals.sun.w;
    let brightness = ambient + (1.0 - ambient) * sunlight;
    return vec4<f32>(in.color * brightness, 1.0);
}
