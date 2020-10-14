use derive_more::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Returns the Chebyshev norm of this point: starting from the origin, how many moves would it
    /// take to get to this point, if you can move diagonally?
    pub fn linf(&self) -> i32 {
        std::cmp::max(self.x, self.y)
    }

    /// Whether these two points are diagonally adjacent.
    pub fn is_adjacent(&self, other: &Point) -> bool {
        std::cmp::max((self.x - other.x).abs(), (self.y - other.y).abs()) <= 1
    }
}
