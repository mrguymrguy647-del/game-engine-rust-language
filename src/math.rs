//! Small math types every 2D game needs: [`Vec2`] and [`Rect`].

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A 2D vector. Used for positions, velocities, directions and sizes.
///
/// `Vec2` is `Copy`, so it is passed around by value just like an `f32`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// Shorthand for [`Vec2::new`]: `vec2(1.0, 2.0)`.
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {
    pub const ZERO: Vec2 = vec2(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// The length (magnitude) of the vector, by Pythagoras.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// A vector pointing the same way with length 1.
    /// Returns [`Vec2::ZERO`] for the zero vector instead of dividing by zero.
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::ZERO
        } else {
            self * (1.0 / len)
        }
    }

    /// The distance between two points.
    pub fn distance(self, other: Vec2) -> f32 {
        (other - self).length()
    }
}

// Operator overloading: implementing these traits is what lets us write
// `a + b`, `a - b`, `v * 2.0`, `-v` and `pos += vel` with vectors.

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        vec2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        vec2(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        vec2(self.x * rhs, self.y * rhs)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        vec2(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

/// An axis-aligned rectangle: `(x, y)` is the top-left corner.
///
/// Screen coordinates grow to the right (x) and *down* (y).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Builds a rectangle of the given size centered on `center`.
    pub fn from_center(center: Vec2, w: f32, h: f32) -> Self {
        Self::new(center.x - w / 2.0, center.y - h / 2.0, w, h)
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> Vec2 {
        vec2(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Axis-aligned bounding box (AABB) collision test: do the two rectangles overlap?
    ///
    /// Two boxes overlap unless one is completely to the left, right,
    /// above or below the other. Touching edges do not count as overlapping.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Is the point inside the rectangle?
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_arithmetic() {
        let a = vec2(1.0, 2.0);
        let b = vec2(3.0, 5.0);
        assert_eq!(a + b, vec2(4.0, 7.0));
        assert_eq!(b - a, vec2(2.0, 3.0));
        assert_eq!(a * 2.0, vec2(2.0, 4.0));
        assert_eq!(-a, vec2(-1.0, -2.0));

        let mut c = a;
        c += b;
        c -= vec2(1.0, 1.0);
        assert_eq!(c, vec2(3.0, 6.0));
    }

    #[test]
    fn length_and_normalize() {
        assert_eq!(vec2(3.0, 4.0).length(), 5.0);
        assert_eq!(vec2(0.0, 10.0).normalized(), vec2(0.0, 1.0));
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
        assert_eq!(vec2(1.0, 1.0).distance(vec2(4.0, 5.0)), 5.0);
    }

    #[test]
    fn rect_overlap() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(a.overlaps(&Rect::new(5.0, 5.0, 10.0, 10.0)));
        assert!(a.overlaps(&Rect::new(2.0, 2.0, 2.0, 2.0))); // fully inside
        assert!(!a.overlaps(&Rect::new(10.0, 0.0, 5.0, 5.0))); // touching edge
        assert!(!a.overlaps(&Rect::new(0.0, 20.0, 5.0, 5.0))); // below
    }

    #[test]
    fn rect_helpers() {
        let r = Rect::from_center(vec2(10.0, 10.0), 4.0, 6.0);
        assert_eq!(r, Rect::new(8.0, 7.0, 4.0, 6.0));
        assert_eq!(r.center(), vec2(10.0, 10.0));
        assert!(r.contains(vec2(8.0, 7.0)));
        assert!(!r.contains(vec2(12.0, 10.0)));
    }
}
