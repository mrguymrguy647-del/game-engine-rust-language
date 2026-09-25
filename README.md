# tiny_engine: a game engine for learning Rust

A small game engine written in Rust, covering **2D graphics, 3D graphics and
physics**, plus five example programs. It was written to be read: about
3,300 lines of commented code (47 tests included), with one dependency
([minifb](https://crates.io/crates/minifb), which opens the window).
Everything else is written from scratch: drawing, text, the 3D pipeline,
the physics solver, input, and random numbers.

**New to Rust? Start with [TUTORIAL.md](TUTORIAL.md)**. It teaches the
language through this engine's code and explains how every part works.

<p>
<img src="docs/world3d.png" width="49%" alt="The 3D playground: crates, balls and a floating pyramid on a checkerboard">
<img src="docs/physics.png" width="49%" alt="The physics sandbox: balls and boxes falling through pegs">
<img src="docs/breakout.png" width="49%" alt="Breakout">
<img src="docs/shapes.png" width="49%" alt="The shapes demo">
</p>

## Quick start

Install Rust from <https://rustup.rs> (1.85 or newer), then:

```sh
cargo run --example world3d    # 3D world with physics: fly around, throw balls
cargo run --example physics    # 2D physics sandbox: click to drop balls and boxes
cargo run --example breakout   # the classic game
cargo run --example hello      # the smallest possible game
cargo run --example shapes     # every 2D drawing function, animated
cargo test                     # run the tests
cargo doc --open               # browse the engine's documentation
```

| Example    | Controls |
|------------|----------|
| `world3d`  | W/A/S/D move · Q/E down/up · Shift faster · arrows or right-drag to look · Space or click throws a ball · B drops a crate · R resets |
| `physics`  | Left click drops a ball · right click drops a box · Space drops a shower · C clears |
| `breakout` | Left/Right or A/D move · Space launches · P pauses · R restarts |
| `hello`    | Arrow keys or WASD move · click to teleport |
| every game | **F3** shows the FPS · **F12** saves a screenshot · Escape quits |

## Features

- **2D drawing:** rectangles, circles, lines, triangles, text in a built-in
  pixel font, color blending, and `.bmp` screenshots.
- **3D rendering, entirely on the CPU:** a camera with ready-made fly
  controls, meshes (cube, pyramid, sphere, checkerboard, or build your
  own), transforms, sunlight shading, a depth buffer, and near-plane
  clipping.
- **Physics in 2D *and* 3D, from the same code:** balls and boxes with
  gravity, bouncing, friction, mass and stacking, plus fixed walls and
  contact reports. (Boxes don't rotate.)
- **Input:** the whole keyboard, and the mouse (position in game pixels,
  buttons, movement and scroll wheel).

## Writing a game

A game is any type that implements the `Game` trait:

```rust
use tiny_engine::prelude::*;

struct MyGame {
    x: f32,
}

impl Game for MyGame {
    fn update(&mut self, ctx: &mut Context) {
        self.x += 50.0 * ctx.dt(); // 50 pixels per second
        if ctx.input.was_pressed(Key::Escape) {
            ctx.quit();
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(Color::BLACK);
        canvas.fill_rect(Rect::new(self.x, 100.0, 16.0, 16.0), Color::YELLOW);
        canvas.draw_text("HELLO!", vec2(8.0, 8.0), 2, Color::WHITE);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tiny_engine::run(Config::default(), MyGame { x: 0.0 })
}
```

Save it as `examples/mygame.rs` and run `cargo run --example mygame`.

### In 3D

A spinning cube you can fly around (WASD and the arrow keys):

```rust
use tiny_engine::prelude::*;

struct Scene {
    camera: Camera3D,
    cube: Mesh,
    time: f32,
}

impl Game for Scene {
    fn update(&mut self, ctx: &mut Context) {
        self.camera.fly(ctx, 3.0);
        self.time += ctx.dt();
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(Color::BLACK);
        let spin = Transform::at(vec3(0.0, 0.0, 3.0)).rotated(vec3(self.time, self.time, 0.0));
        canvas.draw_mesh(&self.cube, &spin, &self.camera);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scene = Scene {
        camera: Camera3D::new(Vec3::ZERO),
        cube: Mesh::cube(Color::ORANGE),
        time: 0.0,
    };
    tiny_engine::run(Config::default(), scene)
}
```

### With physics

```rust
use tiny_engine::physics::{Body, PhysicsWorld};

// 2D, in pixels (y points down). For 3D, pass a Vec3 like vec3(0.0, -9.8, 0.0).
let mut world = PhysicsWorld::new(vec2(0.0, 500.0));
world.add(Body::block(vec2(160.0, 230.0), vec2(320.0, 20.0)).fixed()); // floor
let ball = world.add(Body::ball(vec2(160.0, 20.0), 8.0).with_bounce(0.6));

// In `update`:
world.step(ctx.dt());
// In `draw`: draw the ball at `world.get(ball).unwrap().position`.
```

## What's inside

| File              | What it does                                                        |
|-------------------|---------------------------------------------------------------------|
| `src/engine.rs`   | The `Game` trait, `Config`, `Context` and the game loop (`run`)     |
| `src/canvas.rs`   | Software renderer: pixels, 2D shapes, text, triangles, depth buffer, screenshots |
| `src/render3d.rs` | 3D: `Camera3D`, `Mesh`, `Transform`, lighting, clipping             |
| `src/physics.rs`  | `PhysicsWorld` and `Body`: collisions and an impulse solver, for `Vec2` or `Vec3` |
| `src/input.rs`    | Keyboard and mouse: held, just pressed, just released               |
| `src/math.rs`     | `Vec2`, `Rect`, `Vec3`, and the `Vector` trait that makes physics generic |
| `src/font.rs`     | A built-in 3x5 pixel font                                           |
| `src/color.rs`    | RGB colors and blending                                             |
| `src/rng.rs`      | A tiny xorshift random number generator                             |
| `examples/`       | `hello`, `shapes`, `breakout`, `physics` and `world3d`              |

Only `src/engine.rs` knows minifb exists, so the window library could be
swapped out by rewriting one file.
