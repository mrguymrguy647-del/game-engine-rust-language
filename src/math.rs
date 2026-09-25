//! Vector math: [`Vec2`] and [`Rect`] for 2D, [`Vec3`] for 3D, and the
//! [`Vector`] trait that lets the same code (like the physics engine) work
//! with both.

use std::fmt::Debug;
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

    /// The dot product: `x1 * x2 + y1 * y2`. Positive when the vectors point
    /// roughly the same way, zero when they are at right angles.
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
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

/// A 3D vector.
///
/// In 3D the engine uses these directions: `x` is right, `y` is **up**
/// and `z` is forward (into the screen). Note that `y` points the other way
/// from 2D screen coordinates, where it points down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Shorthand for [`Vec3::new`]: `vec3(1.0, 2.0, 3.0)`.
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Vec3 {
    pub const ZERO: Vec3 = vec3(0.0, 0.0, 0.0);
    pub const ONE: Vec3 = vec3(1.0, 1.0, 1.0);
    pub const UP: Vec3 = vec3(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// A vector pointing the same way with length 1 (or zero for the zero vector).
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::ZERO
        } else {
            self * (1.0 / len)
        }
    }

    pub fn distance(self, other: Vec3) -> f32 {
        (other - self).length()
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// The cross product: a vector at right angles to both inputs. The 3D
    /// renderer uses it to find which way a triangle is facing.
    pub fn cross(self, other: Vec3) -> Vec3 {
        vec3(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Rotates around the x axis (tilting forward/back). A positive angle
    /// turns forward (+z) towards up (+y).
    pub fn rotate_x(self, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        vec3(
            self.x,
            self.y * cos + self.z * sin,
            -self.y * sin + self.z * cos,
        )
    }

    /// Rotates around the y axis (turning left/right). A positive angle
    /// turns forward (+z) towards right (+x).
    pub fn rotate_y(self, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        vec3(
            self.x * cos + self.z * sin,
            self.y,
            -self.x * sin + self.z * cos,
        )
    }

    /// Rotates around the z axis (rolling). A positive angle turns right (+x)
    /// towards up (+y).
    pub fn rotate_z(self, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        vec3(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
            self.z,
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        vec3(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        vec3(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        vec3(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        vec3(-self.x, -self.y, -self.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

/// Anything that behaves like a list of numbers you can add, subtract and
/// scale: [`Vec2`] and [`Vec3`].
///
/// Code written for "any `V: Vector`" works in 2D *and* 3D. The physics
/// engine is written this way, so one implementation handles both.
///
/// The long list after the `:` are *supertraits*: to be a `Vector`, a type
/// must also support `+`, `-`, `* f32` and so on.
pub trait Vector:
    Copy
    + Debug
    + Default
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<f32, Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
{
    /// How many numbers the vector holds: 2 for [`Vec2`], 3 for [`Vec3`].
    const DIMENSIONS: usize;

    /// One component: axis 0 is x, 1 is y, 2 is z.
    fn get(self, axis: usize) -> f32;

    /// A copy of this vector with one component replaced.
    fn with(self, axis: usize, value: f32) -> Self;

    fn dot(self, other: Self) -> f32;

    /// A vector of length 1 along one axis, like `(0, 1)` or `(0, 0, 1)`.
    ///
    /// This one has a *default implementation*, so `Vec2` and `Vec3` get it for free.
    fn unit(axis: usize) -> Self {
        Self::default().with(axis, 1.0)
    }
}

impl Vector for Vec2 {
    const DIMENSIONS: usize = 2;

    fn get(self, axis: usize) -> f32 {
        match axis {
            0 => self.x,
            1 => self.y,
            _ => panic!("Vec2 has no axis {axis}"),
        }
    }

    fn with(mut self, axis: usize, value: f32) -> Self {
        match axis {
            0 => self.x = value,
            1 => self.y = value,
            _ => panic!("Vec2 has no axis {axis}"),
        }
        self
    }

    fn dot(self, other: Self) -> f32 {
        Vec2::dot(self, other)
    }
}

impl Vector for Vec3 {
    const DIMENSIONS: usize = 3;

    fn get(self, axis: usize) -> f32 {
        match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Vec3 has no axis {axis}"),
        }
    }

    fn with(mut self, axis: usize, value: f32) -> Self {
        match axis {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            _ => panic!("Vec3 has no axis {axis}"),
        }
        self
    }

    fn dot(self, other: Self) -> f32 {
        Vec3::dot(self, other)
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

    /// Floats are rarely exactly equal after trigonometry, so compare with a tolerance.
    fn close(a: Vec3, b: Vec3) -> bool {
        a.distance(b) < 1e-5
    }

    #[test]
    fn vec3_arithmetic() {
        let a = vec3(1.0, 2.0, 3.0);
        assert_eq!(a + Vec3::ONE, vec3(2.0, 3.0, 4.0));
        assert_eq!(a - Vec3::ONE, vec3(0.0, 1.0, 2.0));
        assert_eq!(a * 2.0, vec3(2.0, 4.0, 6.0));
        assert_eq!(-a, vec3(-1.0, -2.0, -3.0));
        assert_eq!(a.dot(vec3(1.0, 0.0, 1.0)), 4.0);
        assert_eq!(vec3(0.0, 3.0, 4.0).length(), 5.0);
        assert_eq!(vec3(0.0, 0.0, 9.0).normalized(), vec3(0.0, 0.0, 1.0));
    }

    #[test]
    fn cross_product_is_perpendicular() {
        let x = vec3(1.0, 0.0, 0.0);
        let y = vec3(0.0, 1.0, 0.0);
        assert_eq!(x.cross(y), vec3(0.0, 0.0, 1.0));
        let c = vec3(1.0, 2.0, 3.0).cross(vec3(-4.0, 0.5, 2.0));
        assert!(c.dot(vec3(1.0, 2.0, 3.0)).abs() < 1e-5);
    }

    #[test]
    fn rotations_turn_the_documented_way() {
        let forward = vec3(0.0, 0.0, 1.0);
        let quarter = std::f32::consts::FRAC_PI_2;
        assert!(close(forward.rotate_y(quarter), vec3(1.0, 0.0, 0.0))); // turn right
        assert!(close(forward.rotate_x(quarter), vec3(0.0, 1.0, 0.0))); // tilt up
        assert!(close(vec3(1.0, 0.0, 0.0).rotate_z(quarter), Vec3::UP));
        // Rotating back undoes a rotation.
        let v = vec3(1.0, 2.0, 3.0);
        assert!(close(v.rotate_y(0.7).rotate_y(-0.7), v));
    }

    #[test]
    fn vector_trait_works_for_both_sizes() {
        assert_eq!(Vec2::unit(1), vec2(0.0, 1.0));
        assert_eq!(Vec3::unit(2), vec3(0.0, 0.0, 1.0));
        assert_eq!(vec3(1.0, 2.0, 3.0).with(1, 9.0), vec3(1.0, 9.0, 3.0));
        assert_eq!(vec2(4.0, 5.0).get(0), 4.0);
        assert_eq!(<Vec3 as Vector>::DIMENSIONS, 3);
    }
}
