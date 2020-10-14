use specs::prelude::*;

use crate::components::*;
use crate::event_log::{Event, EventLog};

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AttackIntent>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, QueuedDamage>,
        WriteExpect<'a, EventLog>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (entities, mut intents, stats, mut queues, mut event_log, names): Self::SystemData,
    ) {
        for (entity, intent, stats) in (&entities, &intents, &stats).join() {
            QueuedDamage::add(&mut queues, intent.target, stats.attack);
            event_log.log(
                Event::Damage {
                    from: entity,
                    to: intent.target,
                    amount: stats.attack,
                }
                .format(&names),
            );
        }
        intents.clear();
    }
}
