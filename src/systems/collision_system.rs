use ecs::{ System, DataHelper, EntityIter, EntityData, Entity };
use ecs::system::EntityProcess;

use super::LevelServices;

use components::{LevelComponents, Position, CollisionShape, CollisionAxis};

use na::{self, Pnt2, Iso2, Vec2};
use nc::world::CollisionGroups;
use nc::inspection::Repr;
use nc::bounding_volume::aabb::HasAABB;

use std::sync::Arc;
use std::collections::HashMap;

pub struct CollisionSystem {
    next_uid: usize,
    recycled_uids: Vec<usize>,
    entity_uids: HashMap<Entity, usize>,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {
            next_uid: 0,
            recycled_uids: Vec::new(),
            entity_uids: HashMap::new(),
        }
    }

    fn get_free_uid(&mut self) -> usize {
        match self.recycled_uids.pop() {
            Some(uid) => uid,
            None => { self.next_uid += 1; self.next_uid - 1 },
        }
    }

    fn release_uid(&mut self, uid: usize) {
        self.recycled_uids.push(uid);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CollisionEntityData {
    pub entity: Entity,
    pub axis: CollisionAxis,
}

fn wpos_to_cpos(pos: &Position, shape: &CollisionShape) -> Vec2<f32> {
    let pos = Vec2::new(pos.x / 32.0, pos.y / 32.0);
    match *shape {
        CollisionShape::SingleBox(ref cuboid) => cuboid.aabb(&Iso2::new(pos, na::zero())).center().to_vec(),
        CollisionShape::TwoBoxes{ x: _, y: _ } => unimplemented!(),
    }
}

impl System for CollisionSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    fn activated(&mut self, e: &EntityData<Self::Components>, comps: &Self::Components, services: &mut Self::Services) {
        let uid = self.get_free_uid();

        let pos = &comps.position[*e];
        let coll = &comps.collision[*e];

        let shape = match coll.shape {
            CollisionShape::SingleBox(ref cuboid) => cuboid,
            CollisionShape::TwoBoxes{ x: _, y: _ } => unimplemented!(),
        };

        let data = CollisionEntityData {
            entity: ***e,
            axis: match coll.shape {
                CollisionShape::SingleBox(_) => CollisionAxis::XY,
                CollisionShape::TwoBoxes{ x: _, y: _ } => unimplemented!(),
            }
        };

        services.collision_world.add(uid,
                                     Iso2::new(wpos_to_cpos(pos, &coll.shape), na::zero()),
                                     Arc::new(Box::new(shape.clone()) as Box<Repr<Pnt2<f32>, Iso2<f32>>>),
                                     CollisionGroups::new(),
                                     data);

        self.entity_uids.insert(***e, uid);

        println!("CollisionSystem::activated {:?}", data);
    }

    fn deactivated(&mut self, e: &EntityData<Self::Components>, _: &Self::Components, services: &mut Self::Services) {
        if let Some(uid) = self.entity_uids.remove(&***e) {
            services.collision_world.remove(uid);
            self.release_uid(uid);

            println!("CollisionSystem::deactivated {}", uid);
        }

        println!("ColisionSystem::deactivated: no uid found for entity");
    }
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
        for e in entities {
            let uid = self.entity_uids[&**e];

            let cpos = wpos_to_cpos(&data.position[e], &data.collision[e].shape);

            data.services.collision_world.defered_set_position(uid, Iso2::new(cpos, na::zero()),);
        }

        data.services.collision_world.update();

        let mut contacts = Vec::new();

        data.services.collision_world.contacts(|d1, d2, c| {
            println!("d1: {:?}, d2: {:?}, c: {:?}", d1, d2, c);
            contacts.push((d1.clone(), d2.clone(), c.clone()));
        });

        // d1(index 0) == player, d2(index 1) == wall
        if !contacts.is_empty() {
            data.with_entity_data(&contacts[0].0.entity, | en, comps | {
                let contact = &contacts[0].2;

                // no penetration yet, TODO decide on how to handle this case/look for settings
                if contact.depth <= 0.0 {
                    return;
                }

                let pos = &mut comps.position[en];

                pos.y -= contact.normal.y * contact.depth * 32.0;
                pos.x -= contact.normal.x * contact.depth * 32.0;
            });
        }
    }
}
