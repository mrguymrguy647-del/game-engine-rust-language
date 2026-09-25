//! A tour of everything the canvas can draw, animated with time.
//!
//! Run it with:   cargo run --example shapes
//! Controls:      Space toggles the animation, Escape quits.

use std::f32::consts::TAU;

use tiny_engine::prelude::*;

struct Shapes {
    /// Seconds of animation so far. Only advances while not paused.
    time: f32,
    paused: bool,
}

impl Game for Shapes {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.was_pressed(Key::Escape) {
            ctx.quit();
        }
        if ctx.input.was_pressed(Key::Space) {
            self.paused = !self.paused;
        }
        if !self.paused {
            self.time += ctx.dt();
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.clear(Color::from_hex(0x1B_1B_2F));
        canvas.draw_text("SHAPES", vec2(8.0, 8.0), 2, Color::WHITE);

        // fill_rect + draw_rect: a filled box with an outline around it.
        canvas.fill_rect(Rect::new(20.0, 40.0, 60.0, 40.0), Color::BLUE);
        canvas.draw_rect(Rect::new(16.0, 36.0, 68.0, 48.0), Color::WHITE);
        canvas.draw_text("RECT", vec2(36.0, 88.0), 1, Color::GRAY);

        // fill_circle: a circle that pulses between radius 10 and 24.
        let radius = 17.0 + 7.0 * (self.time * 3.0).sin();
        canvas.fill_circle(vec2(160.0, 60.0), radius, Color::ORANGE);
        canvas.draw_text("CIRCLE", vec2(149.0, 88.0), 1, Color::GRAY);

        // draw_line: a clock hand going round once every 4 seconds.
        let center = vec2(260.0, 60.0);
        let angle = self.time / 4.0 * TAU;
        let tip = center + vec2(angle.sin(), -angle.cos()) * 24.0;
        canvas.draw_line(center, tip, Color::GREEN);
        canvas.fill_circle(center, 2.0, Color::GREEN);
        canvas.draw_text("LINE", vec2(252.0, 88.0), 1, Color::GRAY);

        // set_pixel: a sine wave drawn one pixel at a time.
        for x in 0..canvas.width() as i32 {
            let wave = (x as f32 / 20.0 + self.time * 2.0).sin();
            let y = 140.0 + wave * 20.0;
            canvas.set_pixel(x, y.round() as i32, Color::YELLOW);
        }

        // Color::lerp: a gradient bar blending red into purple.
        for i in 0..32 {
            let t = i as f32 / 31.0;
            let bar = Rect::new(32.0 + i as f32 * 8.0, 180.0, 8.0, 12.0);
            canvas.fill_rect(bar, Color::RED.lerp(Color::PURPLE, t));
        }

        canvas.draw_text_centered("SPACE: PAUSE   ESC: QUIT", 220.0, 1, Color::GRAY);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        title: String::from("Shapes - tiny_engine"),
        ..Config::default()
    };
    let game = Shapes {
        time: 0.0,
        paused: false,
    };
    tiny_engine::run(config, game)
}
