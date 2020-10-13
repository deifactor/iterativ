use crate::{ai, components::*, event_log, map, systems::*};

use log::info;
use specs::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct PlayerId(pub Entity);

#[derive(Debug, Clone)]
pub enum Action {
    Move { dx: i32, dy: i32 },
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
        world.register::<Initiative>();
        world.register::<Ready>();
        world.register::<ai::AIComponent>();
        world.register::<Name>();
        world.insert(LoopState::Looping);
        Engine { world }
    }

    /// Indicates that the player has decided on what they want to do.
    pub fn set_action(&mut self, action: Action) {
        self.world.insert(ai::PlayerAction(action))
    }

    pub fn perform(&mut self, entity: Entity, action: &Action) {
        info!("{:?} performing {:?}", entity, action);
        if entity == self.world.fetch::<PlayerId>().0 {
            // Clear out the player's action, since we're about to execute it.
            self.world.remove::<ai::PlayerAction>();
            // We're no longer waiting for the player.
            self.world.insert::<LoopState>(LoopState::Looping);
        }

        // If `try_perform` returns an alternate action to do, we try that instead, and continue
        // until we completely fail to have an alternate action (because we succeeded).
        let mut maybe_action = Some(action.clone());
        while let Some(action) = maybe_action {
            maybe_action = self.try_perform(entity, &action);
        }
    }

    /// Attempts to perform the given action. If that action fails because of the context, returns
    /// the action that should be tried instead. For example, moving into a square with an entity
    /// will return an Attack action targeting that entity.
    pub fn try_perform(&mut self, entity: Entity, action: &Action) -> Option<Action> {
        match action {
            Action::Move { dx, dy } => {
                let blocker = self.do_move(entity, *dx, *dy)?;
                Some(Action::Attack { target: blocker })
            }
            Action::Attack { target } => {
                self.do_attack(entity, *target);
                None
            }
        }
    }

    /// Tries to move the given entity by the given displacement. If it fails to do so, returns the
    /// entity that was blocking it.
    pub fn do_move(&mut self, entity: Entity, dx: i32, dy: i32) -> Option<Entity> {
        let map = self.world.fetch::<map::Map>();
        let mut pos_storage = self.world.write_storage::<Position>();
        let pos = pos_storage
            .get_mut(entity)
            .expect("can't move something without a position");
        let new_pos = (pos.0.x + dx, pos.0.y + dy);
        // TODO: only do this check if the entity is a collider
        if let Some(blocker) = map.blockers[map.idx(new_pos.0, new_pos.1)] {
            Some(blocker)
        } else {
            pos.0.x += dx;
            pos.0.y += dy;
            None
        }
    }

    fn do_attack(&mut self, entity: Entity, target: Entity) -> () {
        let mut log = self.world.fetch_mut::<event_log::EventLog>();
        log.log(event_log::Event::Damage {
            from: entity,
            to: target,
            amount: 1,
        });
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
                self.perform(entity, &action);
                MapUpdateSystem.run_now(&self.world);
            }
            if *self.world.fetch_mut::<LoopState>() == LoopState::WaitingForPlayer {
                return;
            }
        }
    }
}
