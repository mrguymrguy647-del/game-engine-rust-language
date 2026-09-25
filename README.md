# tiny_engine: a game engine for learning Rust

A small 2D game engine written in Rust, plus three games built on it. It
was written to be read: about 1,200 lines of commented code (tests included) with one
dependency ([minifb](https://crates.io/crates/minifb), which opens the
window). Everything else is written from scratch, including drawing, text,
input, collision and random numbers.

**New to Rust? Start with [TUTORIAL.md](TUTORIAL.md)**. It teaches the
language through this engine's code and explains how the engine works.

![Breakout](docs/breakout.png)

## Quick start

Install Rust from <https://rustup.rs> (1.85 or newer), then:

```sh
cargo run --release --example breakout   # the full game
cargo run --example hello                # the smallest possible game
cargo run --example shapes               # every drawing function, animated
cargo test                               # run the tests
cargo doc --open                         # browse the engine's documentation
```

**Breakout controls:** Left/Right or A/D move · Space launches · P pauses ·
R restarts · Escape quits. In any game, **F12** saves a screenshot as a `.bmp`.

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

## What's inside

| File              | What it does                                                        |
|-------------------|---------------------------------------------------------------------|
| `src/engine.rs`   | The `Game` trait, `Config`, `Context` and the game loop (`run`)     |
| `src/canvas.rs`   | Software renderer: pixels, rectangles, circles, lines, text, BMP screenshots |
| `src/font.rs`     | A built-in 3x5 pixel font                                           |
| `src/input.rs`    | Keyboard state: held, just pressed, just released                   |
| `src/math.rs`     | `Vec2` (with `+ - *` operators) and `Rect` (with overlap tests)     |
| `src/color.rs`    | RGB colors and blending                                             |
| `src/rng.rs`      | A tiny xorshift random number generator                             |
| `examples/`       | `hello`, `shapes` and `breakout`                                    |

The engine draws every pixel on the CPU into a 320x240 buffer, and minifb
scales it up to the window for a pixel-art look. Only `src/engine.rs` knows
minifb exists, so the window library could be swapped out by rewriting one
file.

![Shapes demo](docs/shapes.png)
