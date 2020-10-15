use crate::geometry::WorldVector;
use crate::{Action, Position};
use specs::{prelude::*, Component};

/// A generic trait for objects to decide what they want to do next.
pub trait AI: Send + Sync + std::fmt::Debug {
    /// What does this entity want to do next? Returning `None` means that the entity has nothing
    /// that it wants to do at the moment.
    fn decide(&mut self, world: &World, me: Entity) -> Option<Action>;
}

/// This AI just moves towards its target as fast as possible.
#[derive(Copy, Clone, Debug)]
pub struct Swarm {
    pub target: Entity,
}

impl AI for Swarm {
    fn decide(&mut self, world: &World, me: Entity) -> Option<Action> {
        let pos_component = world.read_component::<Position>();
        let to_target =
            pos_component.get(self.target).unwrap().0 - pos_component.get(me).unwrap().0;
        if to_target.x.abs() <= 1 && to_target.y.abs() <= 1 {
            Some(Action::Attack {
                target: self.target,
            })
        } else {
            Some(Action::Move {
                motion: WorldVector::new(to_target.x.signum(), to_target.y.signum()),
            })
        }
    }
}

/// This "AI" reads from the global PlayerAction resource. It's used so that the player character
/// can use the same AI system as the rest of the entities.
#[derive(Copy, Clone, Debug)]
pub struct PlayerAI;

/// This resources indicates the action the player wants to take. It's updated whenever the player
/// hits an input key, chooses an ability, etc.
#[derive(Debug)]
pub struct PlayerAction(pub Action);

impl AI for PlayerAI {
    fn decide(&mut self, world: &World, _me: Entity) -> Option<Action> {
        world.try_fetch::<PlayerAction>().map(|act| act.0.clone())
    }
}

#[derive(Component, Debug)]
pub struct AIComponent(pub Box<dyn AI>);
