//! A tiny random number generator.

use std::time::{SystemTime, UNIX_EPOCH};

/// A fast pseudo-random number generator (the "xorshift32" algorithm).
///
/// Good enough for games (particle directions, enemy spawns...). It is *not*
/// suitable for anything security-related like passwords or keys.
///
/// The same seed always produces the same sequence, which is handy for tests
/// and for replaying a game exactly.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        // xorshift gets stuck forever at 0, so replace a zero seed with any other number.
        let state = if seed == 0 { 0x9E37_79B9 } else { seed };
        Self { state }
    }

    /// A generator seeded from the clock, so every run of the game is different.
    pub fn from_time() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(1);
        Self::new(nanos)
    }

    /// The next random 32-bit number. The three shift-and-XOR steps scramble the bits.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// A random float in `0.0..1.0` (1.0 itself is never returned).
    pub fn next_f32(&mut self) -> f32 {
        // Keep the top 24 bits: an f32 can represent every 24-bit integer exactly.
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// A random float in `min..max`.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    /// A random integer in `min..max` (`max` excluded). Returns `min` if the range is empty.
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let span = (max as i64 - min as i64) as u64;
        (min as i64 + (self.next_u32() as u64 % span) as i64) as i32
    }

    /// Returns `true` with the given probability (`0.25` = 25% of the time).
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn zero_seed_still_works() {
        let mut rng = Rng::new(0);
        assert_ne!(rng.next_u32(), 0);
    }

    #[test]
    fn ranges_stay_in_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..10_000 {
            let f = rng.range(-2.0, 3.0);
            assert!((-2.0..3.0).contains(&f));
            let i = rng.range_i32(-5, 5);
            assert!((-5..5).contains(&i));
        }
        assert_eq!(rng.range_i32(3, 3), 3);
    }
}
