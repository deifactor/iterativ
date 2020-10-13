use log::info;
use specs::prelude::*;

use crate::components::*;
use crate::engine::LoopState;

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
