//! A 2D physics sandbox: drop balls and boxes onto a board of pegs.
//!
//! Run it with:   cargo run --release --example physics
//! Controls:      left click drops a ball, right click drops a box,
//!                Space drops a shower of balls, C clears, Escape quits.

use std::collections::{HashMap, VecDeque};

use tiny_engine::physics::{Body, BodyId, PhysicsWorld, Shape};
use tiny_engine::prelude::*;

const WIDTH: f32 = 320.0;
const HEIGHT: f32 = 240.0;
const BACKGROUND: Color = Color::from_hex(0x1A_1C_2C);
const WALL_COLOR: Color = Color::from_hex(0x56_5A_75);
const PEG_COLOR: Color = Color::from_hex(0x94_B0_C2);
const PALETTE: [Color; 5] = [
    Color::RED,
    Color::ORANGE,
    Color::YELLOW,
    Color::GREEN,
    Color::BLUE,
];

/// Pixels per second per second. Screen y points down, so gravity is +y.
const GRAVITY: f32 = 400.0;
const WALL_THICKNESS: f32 = 8.0;
/// Oldest bodies are removed once there are more than this many, to keep it fast.
const MAX_DROPPED: usize = 200;
/// How long (in seconds) a peg glows after something touches it.
const GLOW_TIME: f32 = 0.3;

struct Sandbox {
    world: PhysicsWorld<Vec2>,
    /// The bodies players dropped, oldest first.
    dropped: VecDeque<BodyId>,
    /// The color of each dropped body.
    colors: HashMap<BodyId, Color>,
    pegs: Vec<BodyId>,
    /// Seconds of glow left for each peg that was recently hit.
    glow: HashMap<BodyId, f32>,
}

impl Sandbox {
    fn new() -> Self {
        let mut world = PhysicsWorld::new(vec2(0.0, GRAVITY));

        // The floor and walls: fixed blocks.
        let t = WALL_THICKNESS;
        world.add(Body::block(vec2(WIDTH / 2.0, HEIGHT - t / 2.0), vec2(WIDTH, t)).fixed());
        world.add(Body::block(vec2(t / 2.0, HEIGHT / 2.0), vec2(t, HEIGHT)).fixed());
        world.add(Body::block(vec2(WIDTH - t / 2.0, HEIGHT / 2.0), vec2(t, HEIGHT)).fixed());

        // Rows of pegs: fixed balls, every other row shifted by half a gap.
        let mut pegs = Vec::new();
        for row in 0..5 {
            let y = 60.0 + row as f32 * 22.0;
            let shift = if row % 2 == 0 { 0.0 } else { 14.0 };
            let mut x = 30.0 + shift;
            while x < WIDTH - 24.0 {
                // Frictionless, so balls slide off instead of balancing on top.
                let peg = Body::ball(vec2(x, y), 3.0).fixed().with_friction(0.0);
                pegs.push(world.add(peg));
                x += 28.0;
            }
        }

        Self {
            world,
            dropped: VecDeque::new(),
            colors: HashMap::new(),
            pegs,
            glow: HashMap::new(),
        }
    }

    fn drop_body(&mut self, body: Body<Vec2>, rng: &mut Rng) {
        let id = self.world.add(body);
        let color = PALETTE[rng.range_i32(0, PALETTE.len() as i32) as usize];
        self.colors.insert(id, color);
        self.dropped.push_back(id);

        // Too many? Remove the oldest one.
        if self.dropped.len() > MAX_DROPPED {
            if let Some(oldest) = self.dropped.pop_front() {
                self.world.remove(oldest);
                self.colors.remove(&oldest);
            }
        }
    }

    fn drop_ball(&mut self, at: Vec2, rng: &mut Rng) {
        let radius = rng.range(3.5, 6.0);
        let ball = Body::ball(at, radius).with_bounce(0.5);
        self.drop_body(ball, rng);
    }

    fn drop_box(&mut self, at: Vec2, rng: &mut Rng) {
        let size = vec2(rng.range(8.0, 16.0), rng.range(8.0, 16.0));
        self.drop_body(Body::block(at, size).with_friction(0.6), rng);
    }

    fn clear(&mut self) {
        for id in self.dropped.drain(..) {
            self.world.remove(id);
        }
        self.colors.clear();
    }
}

impl Game for Sandbox {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.was_pressed(Key::Escape) {
            ctx.quit();
        }
        if ctx.input.was_pressed(Key::C) {
            self.clear();
        }

        // Drop things where the mouse is (if it's inside the walls).
        let mouse = ctx.input.mouse_position();
        let inside = Rect::new(16.0, 16.0, WIDTH - 32.0, HEIGHT - 40.0);
        if inside.contains(mouse) {
            if ctx.input.was_mouse_pressed(MouseButton::Left) {
                self.drop_ball(mouse, &mut ctx.rng);
            }
            if ctx.input.was_mouse_pressed(MouseButton::Right) {
                self.drop_box(mouse, &mut ctx.rng);
            }
        }
        if ctx.input.was_pressed(Key::Space) {
            for _ in 0..15 {
                let at = vec2(ctx.rng.range(20.0, WIDTH - 20.0), ctx.rng.range(10.0, 40.0));
                self.drop_ball(at, &mut ctx.rng);
            }
        }

        self.world.step(ctx.dt());

        // Light up every peg that something touched.
        for glow in self.glow.values_mut() {
            *glow -= ctx.dt();
        }
        self.glow.retain(|_, time_left| *time_left > 0.0);
        for contact in self.world.contacts() {
            for id in [contact.a, contact.b] {
                if self.pegs.contains(&id) {
                    self.glow.insert(id, GLOW_TIME);
                }
            }
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(BACKGROUND);

        for (id, body) in self.world.bodies() {
            let color = if let Some(&color) = self.colors.get(&id) {
                color
            } else if let Some(&time_left) = self.glow.get(&id) {
                PEG_COLOR.lerp(Color::WHITE, time_left / GLOW_TIME)
            } else if self.pegs.contains(&id) {
                PEG_COLOR
            } else {
                WALL_COLOR
            };

            match body.shape {
                Shape::Ball { radius } => canvas.fill_circle(body.position, radius, color),
                Shape::Block { half_size } => {
                    let size = half_size * 2.0;
                    canvas.fill_rect(Rect::from_center(body.position, size.x, size.y), color);
                }
            }
        }

        canvas.draw_text(
            "LEFT CLICK: BALL   RIGHT CLICK: BOX",
            vec2(14.0, 12.0),
            1,
            Color::WHITE,
        );
        canvas.draw_text("SPACE: SHOWER   C: CLEAR", vec2(14.0, 20.0), 1, Color::GRAY);
        let count = format!("{} BODIES", self.dropped.len());
        let x = WIDTH - 14.0 - Canvas::text_width(&count, 1);
        canvas.draw_text(&count, vec2(x, 20.0), 1, Color::GRAY);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        title: String::from("Physics sandbox - tiny_engine"),
        ..Config::default()
    };
    tiny_engine::run(config, Sandbox::new())
}
