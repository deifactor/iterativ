use specs::prelude::*;

use crate::components::*;
use crate::event_log::{Event, EventLog};

/// Reaps dead entities.
pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatStats>,
        WriteExpect<'a, EventLog>,
    );

    fn run(&mut self, (entities, stats, mut event_log): Self::SystemData) {
        let mut to_die: Vec<Entity> = Vec::new();
        for (entity, stats) in (&entities, &stats).join() {
            if stats.hp <= 0 {
                to_die.push(entity);
            }
        }
        for dead in to_die {
            entities.delete(dead).expect("couldn't delete");
            event_log.log(Event::Death { who: dead });
        }
    }
}
