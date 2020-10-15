//! Types used for geometry. In theory this could be a generic library, but I think it's easier to
//! just roll my own; it's not like performance is important here.
//!
//! Note that all types here are intended for *in-game coordinates* only. For screen coordinates,
//! use quicksilver's types.
pub struct WorldSpace;

pub type Point = euclid::Point2D<i32, WorldSpace>;
pub type Motion = euclid::Vector2D<i32, WorldSpace>;
