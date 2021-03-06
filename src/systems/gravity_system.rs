use ecs::system::EntityProcess;
use ecs::{DataHelper, EntityIter, System};

use super::LevelServices;

use crate::components::LevelComponents;

pub struct GravitySystem;

impl System for GravitySystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl EntityProcess for GravitySystem {
    fn process(
        &mut self,
        entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        let g = data.services.gravity;
        let delta = data.services.delta_time_s;

        for e in entities {
            let gravity = data.gravity[e];
            let velocity = &mut data.velocity[e];
            let d = g * gravity.f * delta;

            velocity.vy -= d;
        }
    }
}
