//! The heart of the engine: the [`Game`] trait and the game loop in [`run`].
//!
//! This is the only file that talks to the window library (`minifb`).
//! Everything else in the engine is plain Rust, so switching to a different
//! window library would mean rewriting just this file.

use std::error::Error;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use minifb::{MouseMode, ScaleMode, Window, WindowOptions};

use crate::canvas::Canvas;
use crate::color::Color;
use crate::input::{Input, Key, MouseButton};
use crate::math::{Rect, Vec2, vec2};
use crate::render3d::Renderer;
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
    /// Show a frames-per-second counter in the corner. F3 toggles it while playing.
    pub show_fps: bool,
    /// What draws 3D: the graphics card if possible (`Auto`), or always the
    /// CPU. The `TINY_ENGINE_RENDERER` environment variable overrides this.
    pub renderer: Renderer,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: String::from("tiny_engine"),
            width: 320,
            height: 240,
            scale: 3,
            target_fps: 60,
            show_fps: false,
            renderer: Renderer::Auto,
        }
    }
}

/// Everything the engine hands to a game during [`Game::update`].
pub struct Context {
    /// The keyboard and mouse.
    pub input: Input,
    /// A random number generator, seeded differently on every run.
    pub rng: Rng,
    dt: f32,
    time: f32,
    frame: u64,
    /// Seconds per frame, averaged over recent frames. 0.0 until measured.
    average_frame_time: f32,
    renderer_name: String,
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
            average_frame_time: 0.0,
            renderer_name: String::new(),
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

    /// Frames per second, averaged over the last second or so.
    pub fn fps(&self) -> f32 {
        if self.average_frame_time > 0.0 {
            1.0 / self.average_frame_time
        } else {
            0.0
        }
    }

    /// What draws 3D: `"CPU"`, or `"GPU: "` plus the graphics card's name.
    /// Known from the second frame on, because the GPU starts up when the
    /// first mesh is drawn.
    pub fn renderer_name(&self) -> &str {
        &self.renderer_name
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

    /// Prepares the context for a new frame. `real_dt` is the true frame
    /// time; the game sees it capped at [`MAX_DT`].
    pub(crate) fn begin_frame(&mut self, real_dt: f32, keys_down: impl IntoIterator<Item = Key>) {
        self.dt = real_dt.min(MAX_DT);
        self.time += self.dt;
        self.frame += 1;
        // The first frame is timed from just before the loop started, so its
        // time is meaningless (almost zero): skip it.
        if self.frame > 1 && real_dt > 0.0 {
            // A moving average: each frame nudges the value 5% towards the
            // latest measurement, so the number on screen doesn't flicker.
            // We average frame *times*, not frames-per-second values: one
            // unusually quick frame would make a per-second average jump.
            self.average_frame_time = if self.average_frame_time == 0.0 {
                real_dt
            } else {
                self.average_frame_time + (real_dt - self.average_frame_time) * 0.05
            };
        }
        self.input.begin_frame(keys_down);
    }
}

/// Opens a window and runs `game` until the window is closed or the game
/// calls [`Context::quit`].
///
/// This is the *game loop*, the core of every game engine. Each frame it:
/// 1. measures how much time has passed,
/// 2. reads the keyboard and mouse,
/// 3. lets the game update itself,
/// 4. lets the game draw itself,
/// 5. shows the result on screen.
///
/// Two keys work in every game: **F3** shows or hides an FPS counter, and
/// **F12** saves a screenshot.
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
    canvas.set_renderer(renderer_from_env().unwrap_or(config.renderer));
    let mut ctx = Context::new(config.width, config.height);
    let mut show_fps = config.show_fps;
    let mut last_frame = Instant::now();

    while window.is_open() && !ctx.quit_requested {
        // 1. Time.
        let now = Instant::now();
        let real_dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        // 2. Input: translate the window library's keys and buttons into our own types.
        let keys = window.get_keys().into_iter().filter_map(convert_key);
        ctx.begin_frame(real_dt, keys);
        read_mouse(&window, &canvas, &mut ctx.input);

        // 3 + 4. The game's turn.
        game.update(&mut ctx);
        game.draw(&mut canvas);
        canvas.flush_3d(); // make sure any 3D from the graphics card is in the pixels
        if ctx.renderer_name != canvas.renderer_name() {
            ctx.renderer_name = canvas.renderer_name().to_string();
        }

        if ctx.input.was_pressed(Key::F3) {
            show_fps = !show_fps;
        }
        if show_fps {
            draw_fps(&mut canvas, ctx.fps());
        }
        if ctx.input.was_pressed(Key::F12) {
            save_screenshot(&canvas);
        }

        // 5. Show the frame. This also collects new keyboard and mouse events from the OS.
        window.update_with_buffer(canvas.pixels(), canvas.width(), canvas.height())?;
    }

    Ok(())
}

fn read_mouse(window: &Window, canvas: &Canvas, input: &mut Input) {
    let position = window.get_mouse_pos(MouseMode::Pass).and_then(|mouse| {
        window_to_canvas(mouse, window.get_size(), (canvas.width(), canvas.height()))
    });

    let buttons = [
        (minifb::MouseButton::Left, MouseButton::Left),
        (minifb::MouseButton::Right, MouseButton::Right),
        (minifb::MouseButton::Middle, MouseButton::Middle),
    ];
    let held = buttons
        .into_iter()
        .filter(|&(theirs, _)| window.get_mouse_down(theirs))
        .map(|(_, ours)| ours);

    // Different systems report different amounts per wheel "click", so
    // `signum` boils it down to just the direction: 1.0, -1.0 or 0.0.
    let scroll = match window.get_scroll_wheel() {
        Some((_, y)) if y != 0.0 => y.signum(),
        _ => 0.0,
    };

    input.update_mouse(position, held, scroll);
}

/// Converts a mouse position in window pixels into canvas pixels.
///
/// The canvas is scaled up as much as fits while keeping its shape, then
/// centered (with black bars filling any leftover space), so we undo the
/// centering and the scaling. Returns `None` if the window has no size.
fn window_to_canvas(
    mouse: (f32, f32),
    window_size: (usize, usize),
    canvas_size: (usize, usize),
) -> Option<Vec2> {
    let (window_w, window_h) = (window_size.0 as f32, window_size.1 as f32);
    let (canvas_w, canvas_h) = (canvas_size.0 as f32, canvas_size.1 as f32);
    let scale = (window_w / canvas_w).min(window_h / canvas_h);
    if scale <= 0.0 {
        return None; // minimized
    }
    let offset_x = (window_w - canvas_w * scale) / 2.0;
    let offset_y = (window_h - canvas_h * scale) / 2.0;
    Some(vec2(
        (mouse.0 - offset_x) / scale,
        (mouse.1 - offset_y) / scale,
    ))
}

/// Reads the `TINY_ENGINE_RENDERER` environment variable, if it's set:
/// `cpu`, `gpu` or `auto`. It lets you switch renderers without changing code.
fn renderer_from_env() -> Option<Renderer> {
    let value = std::env::var("TINY_ENGINE_RENDERER").ok()?;
    match value.to_lowercase().as_str() {
        "cpu" => Some(Renderer::Cpu),
        "gpu" => Some(Renderer::Gpu),
        "auto" => Some(Renderer::Auto),
        other => {
            eprintln!(
                "tiny_engine: ignoring TINY_ENGINE_RENDERER={other:?} (use cpu, gpu or auto)"
            );
            None
        }
    }
}

fn draw_fps(canvas: &mut Canvas, fps: f32) {
    let text = format!("FPS {}", fps.round());
    let y = canvas.height() as f32 - 9.0;
    let background = Rect::new(0.0, y, Canvas::text_width(&text, 1) + 4.0, 9.0);
    canvas.fill_rect(background, Color::BLACK);
    canvas.draw_text(&text, vec2(2.0, y + 2.0), 1, Color::YELLOW);
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
    use minifb::Key as K;

    let key = match key {
        K::A => Key::A,
        K::B => Key::B,
        K::C => Key::C,
        K::D => Key::D,
        K::E => Key::E,
        K::F => Key::F,
        K::G => Key::G,
        K::H => Key::H,
        K::I => Key::I,
        K::J => Key::J,
        K::K => Key::K,
        K::L => Key::L,
        K::M => Key::M,
        K::N => Key::N,
        K::O => Key::O,
        K::P => Key::P,
        K::Q => Key::Q,
        K::R => Key::R,
        K::S => Key::S,
        K::T => Key::T,
        K::U => Key::U,
        K::V => Key::V,
        K::W => Key::W,
        K::X => Key::X,
        K::Y => Key::Y,
        K::Z => Key::Z,
        K::Key0 => Key::Digit0,
        K::Key1 => Key::Digit1,
        K::Key2 => Key::Digit2,
        K::Key3 => Key::Digit3,
        K::Key4 => Key::Digit4,
        K::Key5 => Key::Digit5,
        K::Key6 => Key::Digit6,
        K::Key7 => Key::Digit7,
        K::Key8 => Key::Digit8,
        K::Key9 => Key::Digit9,
        K::Left => Key::Left,
        K::Right => Key::Right,
        K::Up => Key::Up,
        K::Down => Key::Down,
        K::Space => Key::Space,
        K::Enter => Key::Enter,
        K::Escape => Key::Escape,
        K::Tab => Key::Tab,
        K::Backspace => Key::Backspace,
        K::LeftShift => Key::LeftShift,
        K::RightShift => Key::RightShift,
        K::LeftCtrl => Key::LeftCtrl,
        K::RightCtrl => Key::RightCtrl,
        K::LeftAlt => Key::LeftAlt,
        K::RightAlt => Key::RightAlt,
        K::F1 => Key::F1,
        K::F2 => Key::F2,
        K::F3 => Key::F3,
        K::F4 => Key::F4,
        K::F5 => Key::F5,
        K::F6 => Key::F6,
        K::F7 => Key::F7,
        K::F8 => Key::F8,
        K::F9 => Key::F9,
        K::F10 => Key::F10,
        K::F11 => Key::F11,
        K::F12 => Key::F12,
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
        ctx.begin_frame(0.025, []);
        assert_eq!(ctx.dt(), 0.025);
        assert_eq!(ctx.time(), MAX_DT + 0.025); // the first frame was capped
        assert_eq!(ctx.frame(), 2);
        assert_eq!((ctx.width(), ctx.height()), (100.0, 50.0));
        assert!(ctx.input.was_released(Key::Space));
        // Only the second frame counts towards the FPS: 1 / 0.025 = 40.
        assert!((ctx.fps() - 40.0).abs() < 0.01);
    }

    #[test]
    fn fps_ignores_the_near_zero_first_frame() {
        let mut ctx = Context::new(100, 50);
        ctx.begin_frame(0.000_000_05, []); // 50 nanoseconds: "20 million FPS"
        for _ in 0..10 {
            ctx.begin_frame(1.0 / 75.0, []);
        }
        assert!((ctx.fps() - 75.0).abs() < 0.1, "fps = {}", ctx.fps());
    }

    #[test]
    fn keys_are_converted() {
        assert_eq!(convert_key(minifb::Key::Space), Some(Key::Space));
        assert_eq!(convert_key(minifb::Key::Key7), Some(Key::Digit7));
        assert_eq!(convert_key(minifb::Key::Q), Some(Key::Q));
        assert_eq!(convert_key(minifb::Key::NumLock), None);
    }

    #[test]
    fn mouse_maps_from_window_to_canvas() {
        // Window exactly 3x the canvas: just divide by 3.
        let p = window_to_canvas((480.0, 360.0), (960, 720), (320, 240));
        assert_eq!(p, Some(vec2(160.0, 120.0)));

        // A wider window: the canvas is centered with 120-pixel bars on each side.
        let p = window_to_canvas((120.0, 0.0), (1200, 720), (320, 240));
        assert_eq!(p, Some(vec2(0.0, 0.0)));

        // A minimized window has no size.
        assert_eq!(window_to_canvas((5.0, 5.0), (0, 0), (320, 240)), None);
    }
}
