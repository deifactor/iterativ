use crate::geometry::{Motion, Point};
use crate::tiles::TileId;
use specs::{prelude::*, Component};

/// Models turn order. Entities start out with a given initiative, which ticks down by 1 every
/// turn. When it reaches 0, that entity is ready to move.
#[derive(Component, Debug, Copy, Clone)]
pub struct Initiative {
    pub current: i32,
    pub initial: i32,
}

impl Initiative {
    pub fn new(initial: i32) -> Self {
        Initiative {
            current: initial,
            initial,
        }
    }

    /// Ticks down the initiative count. Returns true if the entity is ready (i.e., if its
    /// initiative reached 0). Note that this will also reset the initiative to its initial value.
    pub fn tick(&mut self) -> bool {
        self.current -= 1;
        if self.current <= 0 {
            self.current = self.initial;
            true
        } else {
            false
        }
    }
}

/// A marker component for entities that are ready. The
#[derive(Component, Default, Debug, Copy, Clone)]
#[storage(NullStorage)]
pub struct Ready;

/// Represents the human-readable name of something. Should be all-lowercase (except for proper
/// nouns, of course).
#[derive(Component, Debug, Clone)]
pub struct Name {
    pub name: String,
}

/// The position of a given entity inside the world.
#[derive(Component, Debug, Copy, Clone)]
pub struct Position(pub Point);

/// A tag to indicate that an entity cannot be moved through.
#[derive(Component, Default, Debug, Copy, Clone)]
#[storage(NullStorage)]
pub struct BlocksMovement;

/// This entity has some kind of visual representation.
#[derive(Component, Debug, Clone)]
pub struct Visible {
    pub tile_id: TileId,
}

// All AttackIntent components indicate that the entity is going to attack/etc this turn. These will
// later be resolved by systems.

/// Indicates that this entity wants to attack the target. Resolved by AttackSystem.
#[derive(Component, Copy, Clone, Debug)]
pub struct AttackIntent {
    pub target: Entity,
}

/// Indicates that this entity wants to move. Resolved by MovementSystem. Only makes sense on entities
/// that have a Position.
#[derive(Component, Copy, Clone, Debug)]
pub struct MoveIntent(pub Motion);
