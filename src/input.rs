//! Keyboard input.

use std::collections::HashSet;

/// The keys a game can ask about.
///
/// The engine has its own `Key` type instead of re-using the window library's,
/// so games never depend on which window library the engine uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    W,
    A,
    S,
    D,
    P,
    R,
    Space,
    Enter,
    Escape,
    F12,
}

/// Which keys are held down, and which changed since the last frame.
#[derive(Debug, Default)]
pub struct Input {
    down: HashSet<Key>,
    down_last_frame: HashSet<Key>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called by the engine once per frame with every key that is currently held.
    pub(crate) fn begin_frame(&mut self, keys_down: impl IntoIterator<Item = Key>) {
        // Last frame's "down" becomes this frame's "down_last_frame". `mem::take`
        // moves the set out and leaves an empty one in its place, so nothing is copied.
        self.down_last_frame = std::mem::take(&mut self.down);
        self.down.extend(keys_down);
    }

    /// Is the key held down right now? True on every frame while it is held.
    /// Good for continuous actions like movement.
    pub fn is_down(&self, key: Key) -> bool {
        self.down.contains(&key)
    }

    /// Was the key pressed *this* frame? True for exactly one frame per press.
    /// Good for one-shot actions like jumping, shooting or opening a menu.
    pub fn was_pressed(&self, key: Key) -> bool {
        self.down.contains(&key) && !self.down_last_frame.contains(&key)
    }

    /// Was the key let go this frame?
    pub fn was_released(&self, key: Key) -> bool {
        !self.down.contains(&key) && self.down_last_frame.contains(&key)
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
