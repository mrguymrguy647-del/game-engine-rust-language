//! A 3D playground: fly around, throw balls, and knock over a tower of crates.
//!
//! Run it with:   cargo run --release --example world3d
//! Controls:      W/A/S/D move, Q/E move down/up, Shift goes faster,
//!                arrow keys or drag with the right mouse button to look,
//!                Space or left click throws a ball, B drops a crate,
//!                R resets the scene, Escape quits.

use std::collections::{HashMap, VecDeque};
use std::f32::consts::TAU;

use duckforge::physics::{Body, BodyId, PhysicsWorld, Shape};
use duckforge::prelude::*;

const SKY: Color = Color::from_hex(0x8C_C8_F0);
const GRASS_LIGHT: Color = Color::from_hex(0x6A_A8_4F);
const GRASS_DARK: Color = Color::from_hex(0x5B_94_43);
const SHADOW: Color = Color::from_hex(0x2E_4A_26);
const STONE: Color = Color::from_hex(0xA0_A0_AA);
/// Crates come in slightly different shades, so neighbors are easy to tell apart.
const CRATE_COLORS: [Color; 3] = [
    Color::from_hex(0xC8_8A_46),
    Color::from_hex(0xA8_6E_34),
    Color::from_hex(0xDA_A0_5C),
];
const GOLD: Color = Color::from_hex(0xF0_C0_30);
const BALL_COLORS: [Color; 4] = [Color::RED, Color::YELLOW, Color::BLUE, Color::PURPLE];

/// The ground is FLOOR_SIZE x FLOOR_SIZE meters.
const FLOOR_SIZE: usize = 20;
/// Meters per second per second, pulling down (-y).
const GRAVITY: f32 = 9.8;
const THROW_SPEED: f32 = 14.0;
const MAX_BALLS: usize = 40;

/// How a physics body should be drawn.
#[derive(Debug, Clone, Copy)]
enum Look {
    Ball(usize),  // which color from BALL_COLORS
    Crate(usize), // which color from CRATE_COLORS
    Stone,
}

struct Playground {
    camera: Camera3D,
    world: PhysicsWorld<Vec3>,
    looks: HashMap<BodyId, Look>,
    /// Thrown balls, oldest first, so we can remove the oldest when there are too many.
    balls: VecDeque<BodyId>,
    time: f32,
    // Meshes are built once, then drawn many times with different transforms.
    floor: Mesh,
    crate_meshes: Vec<Mesh>,
    stone_mesh: Mesh,
    ball_meshes: Vec<Mesh>,
    shadow: Mesh,
    pyramid: Mesh,
}

impl Playground {
    fn new() -> Self {
        let mut game = Self {
            camera: Camera3D::looking_at(vec3(0.0, 2.5, -6.0), vec3(0.0, 1.2, 5.0)),
            world: PhysicsWorld::new(vec3(0.0, -GRAVITY, 0.0)),
            looks: HashMap::new(),
            balls: VecDeque::new(),
            time: 0.0,
            floor: Mesh::checkerboard(FLOOR_SIZE, 1.0, GRASS_LIGHT, GRASS_DARK),
            crate_meshes: CRATE_COLORS.iter().map(|&c| Mesh::cube(c)).collect(),
            stone_mesh: Mesh::cube(STONE),
            ball_meshes: BALL_COLORS.iter().map(|&c| Mesh::sphere(c, 12)).collect(),
            shadow: disc(SHADOW, 12),
            pyramid: Mesh::pyramid(GOLD),
        };
        game.build_scene();
        game
    }

    fn build_scene(&mut self) {
        // The ground: a fixed slab whose top is at y = 0. It has no `Look`
        // because the checkerboard mesh draws it.
        let size = FLOOR_SIZE as f32;
        self.world
            .add(Body::block(vec3(0.0, -0.5, 0.0), vec3(size, 1.0, size)).fixed());

        // A stone platform and two pillars.
        self.add(
            Body::block(vec3(4.5, 0.75, 6.0), vec3(3.0, 1.5, 3.0)).fixed(),
            Look::Stone,
        );
        self.add(
            Body::block(vec3(-4.5, 1.5, 7.0), vec3(1.0, 3.0, 1.0)).fixed(),
            Look::Stone,
        );
        self.add(
            Body::block(vec3(-2.5, 1.0, 8.0), vec3(1.0, 2.0, 1.0)).fixed(),
            Look::Stone,
        );

        // A tower of crates: rows of 3, 2 and 1, each row resting on the one below.
        for (row, count) in [3, 2, 1].into_iter().enumerate() {
            for i in 0..count {
                let x = (i as f32 - (count - 1) as f32 / 2.0) * 1.05;
                let y = 0.5 + row as f32 * 1.0;
                self.add_crate(vec3(x, y, 5.0));
            }
        }
        // Two more crates up on the platform.
        self.add_crate(vec3(4.5, 2.0, 6.0));
        self.add_crate(vec3(4.6, 3.0, 6.1));
    }

    fn add(&mut self, body: Body<Vec3>, look: Look) -> BodyId {
        let id = self.world.add(body);
        self.looks.insert(id, look);
        id
    }

    fn add_crate(&mut self, position: Vec3) {
        let shade = self.looks.len() % CRATE_COLORS.len();
        self.add(Body::block(position, Vec3::ONE), Look::Crate(shade));
    }

    fn remove(&mut self, id: BodyId) {
        self.world.remove(id);
        self.looks.remove(&id);
    }

    fn throw_ball(&mut self, rng: &mut Rng) {
        let forward = self.camera.forward();
        let ball = Body::ball(self.camera.position + forward * 0.8, 0.3)
            .with_velocity(forward * THROW_SPEED + Vec3::UP * 1.5)
            .with_bounce(0.5)
            .with_mass(4.0); // heavy enough to knock crates over
        let color = rng.range_i32(0, BALL_COLORS.len() as i32) as usize;
        let id = self.add(ball, Look::Ball(color));
        self.balls.push_back(id);

        if self.balls.len() > MAX_BALLS {
            if let Some(oldest) = self.balls.pop_front() {
                self.remove(oldest);
            }
        }
    }

    fn drop_crate(&mut self) {
        // From the sky, a few meters in front of the camera.
        let ahead = self.camera.position + self.camera.forward() * 5.0;
        self.add_crate(vec3(ahead.x, 8.0, ahead.z));
    }

    /// The height of the highest fixed surface under `point`, to put shadows on.
    fn ground_below(&self, point: Vec3) -> f32 {
        let mut ground: f32 = 0.0;
        for (_, body) in self.world.bodies() {
            if let (true, Shape::Block { half_size }) = (body.is_fixed(), body.shape) {
                let top = body.position.y + half_size.y;
                let above = (point.x - body.position.x).abs() < half_size.x
                    && (point.z - body.position.z).abs() < half_size.z
                    && top <= point.y;
                if above {
                    ground = ground.max(top);
                }
            }
        }
        ground
    }
}

/// A flat circle facing up, 1 unit across. Building a mesh by hand: one
/// vertex in the middle, a ring around it, and a triangle for each slice.
fn disc(color: Color, segments: usize) -> Mesh {
    let mut mesh = Mesh::new();
    let center = mesh.add_vertex(Vec3::ZERO);
    let ring: Vec<usize> = (0..segments)
        .map(|i| {
            let angle = TAU * i as f32 / segments as f32;
            mesh.add_vertex(vec3(angle.cos(), 0.0, angle.sin()) * 0.5)
        })
        .collect();
    for i in 0..segments {
        let next = (i + 1) % segments;
        // Clockwise when seen from above, so the front faces up.
        mesh.add_triangle(center, ring[next], ring[i], color);
    }
    mesh
}

impl Game for Playground {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.was_pressed(Key::Escape) {
            ctx.quit();
        }
        if ctx.input.was_pressed(Key::R) {
            let camera = self.camera;
            *self = Playground::new();
            self.camera = camera; // keep looking from the same spot
        }

        self.camera.fly(ctx, 5.0);

        if ctx.input.was_pressed(Key::Space) || ctx.input.was_mouse_pressed(MouseButton::Left) {
            self.throw_ball(&mut ctx.rng);
        }
        if ctx.input.was_pressed(Key::B) {
            self.drop_crate();
        }

        self.world.step(ctx.dt());

        // Remove anything that fell off the edge of the world.
        let fallen: Vec<BodyId> = self
            .world
            .bodies()
            .filter(|(_, body)| body.position.y < -30.0)
            .map(|(id, _)| id)
            .collect();
        for id in fallen {
            self.remove(id);
            self.balls.retain(|&ball| ball != id);
        }

        self.time += ctx.dt();
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(SKY);

        // Paint far-away ground below the horizon, so the world doesn't look
        // like it ends at the edge of the checkerboard. The horizon is where a
        // point very far ahead, at eye level, appears on screen.
        let yaw = self.camera.yaw;
        let far_ahead = self.camera.position + vec3(yaw.sin(), 0.0, yaw.cos()) * 10_000.0;
        if let Some(horizon) = self.camera.world_to_screen(far_ahead, canvas) {
            let (width, height) = (canvas.width() as f32, canvas.height() as f32);
            canvas.fill_rect(Rect::new(0.0, horizon.y, width, height), GRASS_DARK);
        }

        canvas.draw_mesh(&self.floor, &Transform::default(), &self.camera);

        for (id, body) in self.world.bodies() {
            let Some(&look) = self.looks.get(&id) else {
                continue; // the ground slab: already drawn as the checkerboard
            };

            let (mesh, transform) = match (look, body.shape) {
                (Look::Ball(color), Shape::Ball { radius }) => (
                    &self.ball_meshes[color],
                    Transform::at(body.position).sized(radius * 2.0),
                ),
                (Look::Crate(shade), Shape::Block { half_size }) => (
                    &self.crate_meshes[shade],
                    Transform::at(body.position).scaled(half_size * 2.0),
                ),
                (Look::Stone, Shape::Block { half_size }) => (
                    &self.stone_mesh,
                    Transform::at(body.position).scaled(half_size * 2.0),
                ),
                _ => continue, // a look that doesn't match its shape: skip it
            };
            canvas.draw_mesh(mesh, &transform, &self.camera);

            // A soft round shadow on whatever is below, smaller the higher up it is.
            if !body.is_fixed() {
                let ground = self.ground_below(body.position);
                let height = body.position.y - ground;
                let width = match body.shape {
                    Shape::Ball { radius } => radius * 2.0,
                    Shape::Block { half_size } => half_size.x.max(half_size.z) * 2.2,
                };
                let size = width * (1.0 - height / 10.0).clamp(0.3, 1.0);
                let spot = vec3(body.position.x, ground + 0.01, body.position.z);
                let shadow = Transform::at(spot).scaled(vec3(size, 1.0, size));
                canvas.draw_mesh(&self.shadow, &shadow, &self.camera);
            }
        }

        // A golden pyramid floating above the pillars, spinning and bobbing.
        let bob = (self.time * 2.0).sin() * 0.3;
        let pyramid = Transform::at(vec3(-4.5, 4.2 + bob, 7.0))
            .rotated(vec3(0.0, self.time, 0.0))
            .sized(1.2);
        canvas.draw_mesh(&self.pyramid, &pyramid, &self.camera);

        // A 2D overlay on top of the 3D scene: crosshair and help text.
        let (cx, cy) = (canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0);
        canvas.draw_line(vec2(cx - 4.0, cy), vec2(cx + 4.0, cy), Color::WHITE);
        canvas.draw_line(vec2(cx, cy - 4.0), vec2(cx, cy + 4.0), Color::WHITE);
        canvas.draw_text(
            "WASD: MOVE   Q/E: DOWN/UP   ARROWS/RIGHT-DRAG: LOOK",
            vec2(4.0, 4.0),
            1,
            Color::BLACK,
        );
        canvas.draw_text(
            "SPACE/CLICK: THROW   B: CRATE   R: RESET",
            vec2(4.0, 12.0),
            1,
            Color::BLACK,
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        title: String::from("3D playground - duckforge"),
        ..Config::default()
    };
    duckforge::run(config, Playground::new())
}
