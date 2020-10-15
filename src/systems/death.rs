use specs::prelude::*;

use crate::components::*;
use crate::engine::LoopState;
use crate::event_log::{Event, EventLog};

/// Reaps dead entities.
pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatStats>,
        WriteExpect<'a, EventLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, IsPlayer>,
        WriteExpect<'a, LoopState>,
    );

    fn run(
        &mut self,
        (entities, stats, mut event_log, names, is_player, mut loop_state): Self::SystemData,
    ) {
        let mut to_die: Vec<Entity> = Vec::new();
        for (entity, stats, is_player) in (&entities, &stats, is_player.maybe()).join() {
            if stats.hp <= 0 {
                to_die.push(entity);
                if is_player.is_some() {
                    *loop_state = LoopState::GameOver;
                }
            }
        }
        for dead in to_die {
            entities.delete(dead).expect("couldn't delete");
            event_log.log(Event::Death { who: dead }.format(&names));
        }
    }
}
