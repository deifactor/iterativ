use specs::prelude::*;

use crate::components::*;
use crate::event_log::{Event, EventLog};

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (
        WriteStorage<'a, AttackIntent>,
        Entities<'a>,
        WriteExpect<'a, EventLog>,
    );

    fn run(&mut self, (mut intents, entities, mut event_log): Self::SystemData) {
        for (intent, entity) in (&intents, &entities).join() {
            event_log.log(Event::Damage {
                from: entity,
                to: intent.target,
                amount: 1,
            });
        }
        intents.clear();
    }
}
