use specs::prelude::*;

use crate::components::*;

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, QueuedDamage>,
    );

    fn run(&mut self, (mut stats, mut queues): Self::SystemData) {
        for (stats, queue) in (&mut stats, &queues).join() {
            stats.hp -= queue.0.iter().sum::<i32>();
        }
        queues.clear();
    }
}
