//! # tiny_engine
//!
//! A deliberately small 2D game engine, written for learning Rust.
//!
//! A game is any type that implements the [`Game`](engine::Game) trait.
//! Hand it to [`run`] and the engine opens a window and calls your game's
//! `update` and `draw` methods once per frame:
//!
//! ```no_run
//! use tiny_engine::prelude::*;
//!
//! struct MyGame {
//!     x: f32,
//! }
//!
//! impl Game for MyGame {
//!     fn update(&mut self, ctx: &mut Context) {
//!         self.x += 50.0 * ctx.dt(); // move right at 50 pixels per second
//!     }
//!
//!     fn draw(&self, canvas: &mut Canvas) {
//!         canvas.clear(Color::BLACK);
//!         canvas.fill_rect(Rect::new(self.x, 100.0, 16.0, 16.0), Color::YELLOW);
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tiny_engine::run(Config::default(), MyGame { x: 0.0 })
//! }
//! ```

pub mod canvas;
pub mod color;
pub mod engine;
pub mod input;
pub mod math;
pub mod rng;

// Not `pub`: the font is an internal detail of `Canvas::draw_text`.
mod font;

pub use engine::run;

/// Everything a game usually needs, importable in one line:
/// `use tiny_engine::prelude::*;`
pub mod prelude {
    pub use crate::canvas::Canvas;
    pub use crate::color::Color;
    pub use crate::engine::{Config, Context, Game};
    pub use crate::input::{Input, Key};
    pub use crate::math::{Rect, Vec2, vec2};
    pub use crate::rng::Rng;
}
