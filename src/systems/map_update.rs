use specs::prelude::*;

use crate::components::*;
use crate::map::Map;

pub struct MapUpdateSystem;

impl<'a> System<'a> for MapUpdateSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksMovement>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, positions, blocked, entities): Self::SystemData) {
        map.clear_entities();
        for (position, blocked, entity) in (&positions, blocked.maybe(), &entities).join() {
            let idx = map.idx(position.0.x, position.0.y);
            if blocked.is_some() {
                map.blockers[idx] = Some(entity);
            }
            map.entities[idx].push(entity);
        }
    }
}
