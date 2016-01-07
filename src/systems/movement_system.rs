use ecs::{System, DataHelper, EntityIter};
use ecs::system::EntityProcess;

use super::LevelServices;

use components::LevelComponents;
use application::InputIntent;

use na::{self, Vec2};

use num::traits::Zero;

pub struct MovementSystem;

impl System for MovementSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

fn add_clamp_to_zero(val: f32, add: f32, minmax: f32) -> f32 {
    if val < 0.0 {
        na::clamp(val + add, -minmax, 0.0)
    } else {
        na::clamp(val - add, 0.0, minmax)
    }
}

impl EntityProcess for MovementSystem {
    fn process(&mut self,
               entities: EntityIter<LevelComponents>,
               data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let (move_left, move_right) = {
                let intents = &data.intents[e];

                (intents.contains(&InputIntent::MoveLeft),
                 intents.contains(&InputIntent::MoveRight))
            };

            let delta = data.services.delta_time_s;

            let vel = {
                let movement = &mut data.movement[e];

                if move_left == move_right {
                    if movement.vel.is_zero() {
                        continue;
                    }

                    // TODO reduce speed instead(e.g. by acc)
                    movement.vel.x = add_clamp_to_zero(movement.vel.x,
                                                       movement.acc.x * delta,
                                                       movement.max_vel.x);
                    movement.vel.y = add_clamp_to_zero(movement.vel.y,
                                                       movement.acc.y * delta,
                                                       movement.max_vel.y);
                } else {
                    let acc = if move_left {
                        -movement.acc
                    } else {
                        movement.acc
                    } * delta;

                    movement.vel = na::partial_clamp(&(movement.vel + acc),
                                                     &-movement.max_vel,
                                                     &movement.max_vel)
                                       .unwrap_or(&Vec2::zero())
                                       .clone();
                }

                movement.vel.clone()
            };

            {
                let velocity = &mut data.velocity[e];
                velocity.vx += delta * vel.x;
                velocity.vy += delta * vel.y;
            }
        }
    }
}
