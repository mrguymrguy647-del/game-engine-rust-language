//! The heart of the engine: the [`Game`] trait and the game loop in [`run`].
//!
//! This is the only file that talks to the window library (`minifb`).
//! Everything else in the engine is plain Rust, so switching to a different
//! window library would mean rewriting just this file.

use std::error::Error;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use minifb::{ScaleMode, Window, WindowOptions};

use crate::canvas::Canvas;
use crate::input::{Input, Key};
use crate::rng::Rng;

/// The longest time step a game will ever see, in seconds.
///
/// If the game freezes for a moment (say, while the window is being dragged)
/// the next frame would otherwise get a huge `dt`, and fast objects would
/// teleport straight through walls.
const MAX_DT: f32 = 1.0 / 30.0;

/// What every game must provide. The engine calls both methods once per frame.
pub trait Game {
    /// Advance the game by one step: read input, move things, check collisions.
    fn update(&mut self, ctx: &mut Context);

    /// Draw the current state of the game onto the canvas.
    ///
    /// This takes `&self`, not `&mut self`: drawing can look at the game but
    /// never change it, and the compiler enforces that for us.
    fn draw(&self, canvas: &mut Canvas);
}

/// Settings for the window and the game loop.
#[derive(Debug, Clone)]
pub struct Config {
    /// Text in the window's title bar.
    pub title: String,
    /// Width of the canvas in pixels.
    pub width: usize,
    /// Height of the canvas in pixels.
    pub height: usize,
    /// The window starts `scale` times bigger than the canvas, for a chunky
    /// pixel-art look. The window can also be resized freely.
    pub scale: usize,
    /// How many frames per second the loop aims for.
    pub target_fps: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: String::from("tiny_engine"),
            width: 320,
            height: 240,
            scale: 3,
            target_fps: 60,
        }
    }
}

/// Everything the engine hands to a game during [`Game::update`].
pub struct Context {
    /// The keyboard.
    pub input: Input,
    /// A random number generator, seeded differently on every run.
    pub rng: Rng,
    dt: f32,
    time: f32,
    frame: u64,
    width: usize,
    height: usize,
    quit_requested: bool,
}

impl Context {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            input: Input::new(),
            rng: Rng::from_time(),
            dt: 0.0,
            time: 0.0,
            frame: 0,
            width,
            height,
            quit_requested: false,
        }
    }

    /// Seconds since the previous frame ("delta time"). Multiply speeds by
    /// this so things move at the same speed no matter the frame rate.
    pub fn dt(&self) -> f32 {
        self.dt
    }

    /// Seconds since the game started.
    pub fn time(&self) -> f32 {
        self.time
    }

    /// How many frames have run so far, starting at 1.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Width of the canvas in pixels.
    pub fn width(&self) -> f32 {
        self.width as f32
    }

    /// Height of the canvas in pixels.
    pub fn height(&self) -> f32 {
        self.height as f32
    }

    /// Ask the engine to close the window at the end of this frame.
    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    /// Prepares the context for a new frame.
    pub(crate) fn begin_frame(&mut self, dt: f32, keys_down: impl IntoIterator<Item = Key>) {
        self.dt = dt;
        self.time += dt;
        self.frame += 1;
        self.input.begin_frame(keys_down);
    }
}

/// Opens a window and runs `game` until the window is closed or the game
/// calls [`Context::quit`]. Press F12 at any time to save a screenshot.
///
/// This is the *game loop*, the core of every game engine. Each frame it:
/// 1. measures how much time has passed,
/// 2. reads the keyboard,
/// 3. lets the game update itself,
/// 4. lets the game draw itself,
/// 5. shows the result on screen.
pub fn run<G: Game>(config: Config, mut game: G) -> Result<(), Box<dyn Error>> {
    let options = WindowOptions {
        resize: true,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        &config.title,
        config.width * config.scale,
        config.height * config.scale,
        options,
    )?;
    // minifb waits inside `update_with_buffer` so we don't run faster than this.
    window.set_target_fps(config.target_fps);

    let mut canvas = Canvas::new(config.width, config.height);
    let mut ctx = Context::new(config.width, config.height);
    let mut last_frame = Instant::now();

    while window.is_open() && !ctx.quit_requested {
        // 1. Time.
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32().min(MAX_DT);
        last_frame = now;

        // 2. Input: translate the window library's keys into our own `Key`s.
        let keys = window.get_keys().into_iter().filter_map(convert_key);
        ctx.begin_frame(dt, keys);

        // 3 + 4. The game's turn.
        game.update(&mut ctx);
        game.draw(&mut canvas);

        if ctx.input.was_pressed(Key::F12) {
            save_screenshot(&canvas);
        }

        // 5. Show the frame. This also collects new keyboard events from the OS.
        window.update_with_buffer(canvas.pixels(), canvas.width(), canvas.height())?;
    }

    Ok(())
}

fn save_screenshot(canvas: &Canvas) {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = format!("screenshot-{millis}.bmp");

    // A failed screenshot shouldn't crash the game, so we report the error
    // instead of passing it up with `?`.
    match canvas.save_bmp(&path) {
        Ok(()) => println!("Saved {path}"),
        Err(err) => eprintln!("Could not save {path}: {err}"),
    }
}

/// Maps a `minifb` key to our own [`Key`], or `None` for keys games can't ask about.
fn convert_key(key: minifb::Key) -> Option<Key> {
    let key = match key {
        minifb::Key::Left => Key::Left,
        minifb::Key::Right => Key::Right,
        minifb::Key::Up => Key::Up,
        minifb::Key::Down => Key::Down,
        minifb::Key::W => Key::W,
        minifb::Key::A => Key::A,
        minifb::Key::S => Key::S,
        minifb::Key::D => Key::D,
        minifb::Key::P => Key::P,
        minifb::Key::R => Key::R,
        minifb::Key::Space => Key::Space,
        minifb::Key::Enter => Key::Enter,
        minifb::Key::Escape => Key::Escape,
        minifb::Key::F12 => Key::F12,
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_tracks_time_and_frames() {
        let mut ctx = Context::new(100, 50);
        ctx.begin_frame(0.5, [Key::Space]);
        ctx.begin_frame(0.25, []);
        assert_eq!(ctx.dt(), 0.25);
        assert_eq!(ctx.time(), 0.75);
        assert_eq!(ctx.frame(), 2);
        assert_eq!((ctx.width(), ctx.height()), (100.0, 50.0));
        assert!(ctx.input.was_released(Key::Space));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        assert_eq!(convert_key(minifb::Key::Space), Some(Key::Space));
        assert_eq!(convert_key(minifb::Key::Q), None);
    }
}
