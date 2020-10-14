use log::info;
use specs::prelude::*;

use crate::ai;
use crate::components::*;
use crate::geometry::Motion;
use crate::map::Map;
use crate::systems::*;

#[derive(Debug, Copy, Clone)]
pub struct PlayerId(pub Entity);

#[derive(Debug, Clone)]
pub enum Action {
    Move { motion: Motion },
    Attack { target: Entity },
}

pub struct Engine {
    pub world: World,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LoopState {
    Looping,
    WaitingForPlayer,
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<BlocksMovement>();
        world.register::<Visible>();
        specs::RunNow::setup(&mut InitiativeSystem, &mut world);
        world.register::<Ready>();
        world.register::<ai::AIComponent>();
        world.register::<Name>();
        world.register::<MoveIntent>();
        world.register::<AttackIntent>();
        world.insert(LoopState::Looping);
        Engine { world }
    }

    pub fn set_action(&mut self, action: Action) {
        if let Some(action) = self.normalize_action(action) {
            self.world.insert(ai::PlayerAction(action));
        }
    }

    /// Translates a 'high-level' action into a low-level one. This implements things like moving
    /// into an entity to attack it.
    ///
    /// If this returns `None`, then the action requested is impossible (e.g., moving into a wall).
    fn normalize_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Move { motion } => {
                let map = self.world.fetch::<Map>();
                let player_id = self.world.fetch::<PlayerId>().0;
                let player_pos = self
                    .world
                    .read_storage::<Position>()
                    .get(player_id)
                    .unwrap()
                    .0;
                let target = player_pos + motion;
                if let Some(blocker) = map.blockers[map.idx(target.x, target.y)] {
                    Some(Action::Attack { target: blocker })
                } else {
                    Some(action)
                }
            }
            _ => Some(action),
        }
    }

    pub fn perform(&mut self, entity: Entity, action: Action) {
        info!("{:?} performing {:?}", entity, action);
        if entity == self.world.fetch::<PlayerId>().0 {
            // Clear out the player's action, since we're about to execute it.
            self.world.remove::<ai::PlayerAction>();
            // We're no longer waiting for the player.
            self.world.insert::<LoopState>(LoopState::Looping);
        }

        match action {
            Action::Move { motion } => {
                self.world
                    .write_storage::<MoveIntent>()
                    .insert(entity, MoveIntent(motion))
                    .expect("failed to insert move intent");
            }
            Action::Attack { target } => {
                self.world
                    .write_storage::<AttackIntent>()
                    .insert(entity, AttackIntent { target })
                    .expect("failed to insert attack intent");
            }
        }
    }

    fn find_actor(&self) -> Option<(Entity, Action)> {
        let ready = self.world.write_storage::<Ready>();
        let mut ai = self.world.write_storage::<ai::AIComponent>();
        let entity = self.world.entities();
        let player = self.world.fetch::<PlayerId>().0;
        for (_ready, ai, entity) in (&ready, &mut ai, &entity).join() {
            if let Some(action) = ai.0.decide(&self.world, entity) {
                return Some((entity, action));
            } else if entity == player {
                *self.world.fetch_mut::<LoopState>() = LoopState::WaitingForPlayer;
                return None;
            }
        }
        None
    }

    // The game loop works like this: we find an entity that has something that it can do. If at
    // any point the player is ready but doesn't have anything it can do, we enter the 'waiting for
    // player' state. In this state, the initiative system does nothing. And even if there are mobs
    // that are also ready, we'll eventually execute their actions given enough runs through
    // tick(), meaning that we'll get into a situation where the player is the only mob that's
    // ready, but it's not doing anything; i.e., we're waiting for playre input.
    //
    // Then, when the player inputs something, the next tick through will finally have the player's
    // "AI" return an action, causing us to leave `WaitingForPlayer` and resume normal engine
    // execution.

    pub fn tick(&mut self) {
        // We run this in a loop so that we're not stuck ticking once per frame. TODO: Move the
        // tick-until-waiting into a separate function or something.
        loop {
            InitiativeSystem.run_now(&self.world);
            if let Some((entity, action)) = self.find_actor() {
                self.world.write_storage::<Ready>().remove(entity);
                self.perform(entity, action);
                MapUpdateSystem.run_now(&self.world);
            }
            MovementSystem.run_now(&self.world);
            AttackSystem.run_now(&self.world);
            if *self.world.fetch_mut::<LoopState>() == LoopState::WaitingForPlayer {
                return;
            }
        }
    }
}
