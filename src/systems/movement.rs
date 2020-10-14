use specs::prelude::*;

use crate::components::*;
use crate::map::Map;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, Position>,
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, (mut intents, mut positions, map): Self::SystemData) {
        for (intent, position) in (&intents, &mut positions).join() {
            let new_pos = position.0 + intent.0;
            if let Some(blocker) = map.blockers[map.idx(new_pos.x, new_pos.y)] {
            } else {
                position.0 = new_pos;
            }
        }
        intents.clear();
    }
}
