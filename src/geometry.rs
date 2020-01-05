use derive_more::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Add, Sub)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}
