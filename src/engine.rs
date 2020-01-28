use crate::geometry::Point;
use crate::{ai, map, tiles};

use log::info;
use specs::{prelude::*, Component};

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

#[derive(Component, Default, Debug, Copy, Clone)]
#[storage(NullStorage)]
pub struct Ready;

pub struct InitiativeSystem;

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        WriteStorage<'a, Ready>,
        ReadExpect<'a, LoopState>,
        Entities<'a>,
    );

    fn run(&mut self, (mut initiative, mut turn, loop_state, entities): Self::SystemData) {
        if *loop_state != LoopState::Looping {
            return;
        }

        for (initiative, entity) in (&mut initiative, &entities).join() {
            if initiative.tick() {
                turn.insert(entity, Ready)
                    .expect("can't set Ready component");
                info!("entity {:?} is ready", entity);
            }
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct PlayerId(pub Entity);

#[derive(Component, Debug, Copy, Clone)]
pub struct Position(pub Point<i32>);

#[derive(Debug, Clone)]
pub enum Action {
    Move { dx: i32, dy: i32 },
}

#[derive(Component, Default, Debug, Copy, Clone)]
/// Tagged to indicate that an entity cannot be moved through.
pub struct BlocksMovement;

pub struct MapUpdateSystem;

impl<'a> System<'a> for MapUpdateSystem {
    type SystemData = (
        WriteExpect<'a, map::Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksMovement>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, positions, blocked, entities): Self::SystemData) {
        map.clear_entities();
        for (position, blocked, entity) in (&positions, blocked.maybe(), &entities).join() {
            let idx = map.idx(position.0.x, position.0.y);
            if blocked.is_some() {
                map.blocked_by_entity[idx] = true;
            }
            map.entities[idx].push(entity);
        }
    }
}

#[derive(Component, Debug, Clone)]
/// If something should be drawn in the world, what's its tile? This is *not* the underlying Image,
/// since that's wrapped in an Rc and so it's not thread-safe.
pub struct Visible {
    pub tile_id: tiles::TileId,
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
        match action {
            Action::Move { dx, dy } => {
                let map = self.world.fetch::<map::Map>();
                let mut pos_storage = self.world.write_storage::<Position>();
                let pos = pos_storage
                    .get_mut(entity)
                    .expect("can't move something without a position");
                let new_pos = (pos.0.x + dx, pos.0.y + dy);
                if !map.blocked(new_pos.0, new_pos.1) {
                    pos.0.x += dx;
                    pos.0.y += dy;
                }
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
        InitiativeSystem.run_now(&self.world);
        if let Some((entity, action)) = self.find_actor() {
            self.world.write_storage::<Ready>().remove(entity);
            self.perform(entity, &action);
            MapUpdateSystem.run_now(&self.world);
        }
    }
}
