//! Keyboard and mouse input.

use std::collections::HashSet;
use std::hash::Hash;

use crate::math::Vec2;

/// A key on the keyboard.
///
/// The engine has its own `Key` type instead of re-using the window library's,
/// so games never depend on which window library the engine uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// The number keys along the top of the keyboard. (A name can't start
    /// with a digit in Rust, so it's `Digit1`, not `1`.)
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Left,
    Right,
    Up,
    Down,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    F1,
    F2,
    /// The engine uses F3 to show and hide the FPS counter.
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    /// The engine uses F12 to save a screenshot.
    F12,
}

/// A button on the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Remembers which buttons are held this frame and which were held last
/// frame, so it can tell "held" apart from "just pressed".
///
/// It's *generic* over `T`, the kind of button: the same code tracks keyboard
/// keys (`ButtonState<Key>`) and mouse buttons (`ButtonState<MouseButton>`).
#[derive(Debug)]
struct ButtonState<T> {
    down: HashSet<T>,
    down_last_frame: HashSet<T>,
}

// `#[derive(Default)]` would require `T: Default`, which keys don't need,
// so we write this one by hand.
impl<T> Default for ButtonState<T> {
    fn default() -> Self {
        Self {
            down: HashSet::new(),
            down_last_frame: HashSet::new(),
        }
    }
}

impl<T: Copy + Eq + Hash> ButtonState<T> {
    fn begin_frame(&mut self, held: impl IntoIterator<Item = T>) {
        // Last frame's "down" becomes this frame's "down_last_frame". `mem::take`
        // moves the set out and leaves an empty one in its place, so nothing is copied.
        self.down_last_frame = std::mem::take(&mut self.down);
        self.down.extend(held);
    }

    fn is_down(&self, button: T) -> bool {
        self.down.contains(&button)
    }

    fn was_pressed(&self, button: T) -> bool {
        self.down.contains(&button) && !self.down_last_frame.contains(&button)
    }

    fn was_released(&self, button: T) -> bool {
        !self.down.contains(&button) && self.down_last_frame.contains(&button)
    }
}

/// Everything about the keyboard and mouse this frame.
#[derive(Debug, Default)]
pub struct Input {
    keys: ButtonState<Key>,
    mouse_buttons: ButtonState<MouseButton>,
    /// `None` until the mouse has been over the window at least once.
    mouse_position: Option<Vec2>,
    mouse_delta: Vec2,
    scroll: f32,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called by the engine once per frame with every key that is currently held.
    pub(crate) fn begin_frame(&mut self, keys_down: impl IntoIterator<Item = Key>) {
        self.keys.begin_frame(keys_down);
    }

    /// Called by the engine once per frame with the mouse's state.
    /// `position` is in canvas pixels, or `None` if it isn't known.
    pub(crate) fn update_mouse(
        &mut self,
        position: Option<Vec2>,
        buttons_down: impl IntoIterator<Item = MouseButton>,
        scroll: f32,
    ) {
        self.mouse_buttons.begin_frame(buttons_down);
        self.mouse_delta = match (self.mouse_position, position) {
            (Some(old), Some(new)) => new - old,
            _ => Vec2::ZERO, // no movement until we know two positions
        };
        if position.is_some() {
            self.mouse_position = position;
        }
        self.scroll = scroll;
    }

    // ---- Keyboard ----

    /// Is the key held down right now? True on every frame while it is held.
    /// Good for continuous actions like movement.
    pub fn is_down(&self, key: Key) -> bool {
        self.keys.is_down(key)
    }

    /// Was the key pressed *this* frame? True for exactly one frame per press.
    /// Good for one-shot actions like jumping, shooting or opening a menu.
    pub fn was_pressed(&self, key: Key) -> bool {
        self.keys.was_pressed(key)
    }

    /// Was the key let go this frame?
    pub fn was_released(&self, key: Key) -> bool {
        self.keys.was_released(key)
    }

    /// Turns two keys into a direction: -1.0 if only `negative` is held,
    /// 1.0 if only `positive` is held, and 0.0 if neither or both are.
    ///
    /// `input.axis(Key::Left, Key::Right)` is a handy way to read left/right movement.
    pub fn axis(&self, negative: Key, positive: Key) -> f32 {
        let mut value = 0.0;
        if self.is_down(negative) {
            value -= 1.0;
        }
        if self.is_down(positive) {
            value += 1.0;
        }
        value
    }

    // ---- Mouse ----

    /// Where the mouse pointer is, in canvas pixels (the same coordinates you
    /// draw with). It can be outside the canvas if the pointer is outside the
    /// game area.
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position.unwrap_or(Vec2::ZERO)
    }

    /// How far the mouse moved since last frame, in canvas pixels.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Is the mouse button held down right now?
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.is_down(button)
    }

    /// Was the mouse button clicked this frame? True for exactly one frame per click.
    pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.was_pressed(button)
    }

    /// Was the mouse button let go this frame?
    pub fn was_mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons.was_released(button)
    }

    /// The scroll wheel this frame: `1.0` rolled up (away from you), `-1.0`
    /// rolled down, `0.0` not moved.
    pub fn scroll(&self) -> f32 {
        self.scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vec2;

    #[test]
    fn pressed_is_true_for_one_frame_only() {
        let mut input = Input::new();

        input.begin_frame([Key::Space]);
        assert!(input.is_down(Key::Space));
        assert!(input.was_pressed(Key::Space));

        input.begin_frame([Key::Space]);
        assert!(input.is_down(Key::Space));
        assert!(!input.was_pressed(Key::Space));

        input.begin_frame([]);
        assert!(!input.is_down(Key::Space));
        assert!(input.was_released(Key::Space));
    }

    #[test]
    fn axis_combines_two_keys() {
        let mut input = Input::new();
        input.begin_frame([Key::Left]);
        assert_eq!(input.axis(Key::Left, Key::Right), -1.0);
        input.begin_frame([Key::Right]);
        assert_eq!(input.axis(Key::Left, Key::Right), 1.0);
        input.begin_frame([Key::Left, Key::Right]);
        assert_eq!(input.axis(Key::Left, Key::Right), 0.0);
    }

    #[test]
    fn mouse_clicks_and_movement() {
        let mut input = Input::new();

        // The first known position doesn't count as movement.
        input.update_mouse(Some(vec2(10.0, 20.0)), [MouseButton::Left], 0.0);
        assert_eq!(input.mouse_position(), vec2(10.0, 20.0));
        assert_eq!(input.mouse_delta(), Vec2::ZERO);
        assert!(input.was_mouse_pressed(MouseButton::Left));

        input.update_mouse(Some(vec2(15.0, 18.0)), [MouseButton::Left], 1.0);
        assert_eq!(input.mouse_delta(), vec2(5.0, -2.0));
        assert!(input.is_mouse_down(MouseButton::Left));
        assert!(!input.was_mouse_pressed(MouseButton::Left));
        assert_eq!(input.scroll(), 1.0);

        // Pointer left the window: keep the last position, no movement.
        input.update_mouse(None, [], 0.0);
        assert_eq!(input.mouse_position(), vec2(15.0, 18.0));
        assert_eq!(input.mouse_delta(), Vec2::ZERO);
        assert!(input.was_mouse_released(MouseButton::Left));
    }
}
