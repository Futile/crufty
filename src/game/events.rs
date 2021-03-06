use ecs::{DataHelper, Entity};

use crate::game;
use crate::systems::interaction_system;
use crate::{components::LevelComponents, systems::LevelServices};

pub trait EventReceiver<T> {
    fn receive_event(&mut self, event: T);
}

#[derive(Copy, Clone, Debug)]
pub struct CollisionStarted {
    pub collider: Entity,
    pub collided: Entity,
}

#[derive(Copy, Clone, Debug)]
pub struct CollisionEnded {
    pub collider: Entity,
    pub collided: Entity,
}

#[derive(Copy, Clone, Debug)]
pub struct InteractionDone {
    pub interactor: Entity,
    pub interacted: Entity,
    pub interaction: game::Interaction,
}

impl EventReceiver<CollisionStarted> for DataHelper<LevelComponents, LevelServices> {
    fn receive_event(&mut self, event: CollisionStarted) {
        interaction_system::on_collision_started(self, &event);
    }
}

impl EventReceiver<CollisionEnded> for DataHelper<LevelComponents, LevelServices> {
    fn receive_event(&mut self, event: CollisionEnded) {
        interaction_system::on_collision_ended(self, &event);
    }
}

impl EventReceiver<InteractionDone> for DataHelper<LevelComponents, LevelServices> {
    fn receive_event(&mut self, _event: InteractionDone) {
        dbg!(_event);
    }
}
