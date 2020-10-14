//! Types used for geometry. In theory this could be a generic library, but I think it's easier to
//! just roll my own; it's not like performance is important here.
//!
//! Note that all types here are intended for *in-game coordinates* only. For screen coordinates,
//! use quicksilver's types.
use derive_more::{Add, AddAssign, Sub, SubAssign};
use std::ops::{Add, AddAssign, Sub};

/// This represents a point in space. You can subtract two poitns to get a motion, or add a motion
/// to a point, but you *cannot* add two points.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// A displacement between two points. You can add or subtract these freely.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Add, AddAssign, Sub, SubAssign)]
pub struct Motion {
    pub dx: i32,
    pub dy: i32,
}

impl Motion {
    /// The Chebyshev norm of this motion: how many moves would it take?
    pub fn linf(&self) -> i32 {
        self.dx.max(self.dy)
    }

    /// Converts each of the coordinates to -1, 0, or +1 according to its sign.
    pub fn signum(&self) -> Motion {
        Motion {
            dx: self.dx.signum(),
            dy: self.dy.signum(),
        }
    }
}

impl Add<Motion> for Point {
    type Output = Self;

    fn add(self, other: Motion) -> Self {
        Self {
            x: self.x + other.dx,
            y: self.y + other.dy,
        }
    }
}

impl AddAssign<Motion> for Point {
    fn add_assign(&mut self, other: Motion) {
        self.x += other.dx;
        self.y += other.dy;
    }
}

impl Sub<Motion> for Point {
    type Output = Self;

    fn sub(self, other: Motion) -> Self {
        Self {
            x: self.x - other.dx,
            y: self.y - other.dy,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Motion;
    fn sub(self, other: Point) -> Motion {
        Motion {
            dx: self.x - other.x,
            dy: self.y - other.y,
        }
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Motion {
    fn from((dx, dy): (i32, i32)) -> Self {
        Self { dx, dy }
    }
}
