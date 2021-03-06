use std::collections::{HashMap, HashSet};

use ecs::Entity;
use smallvec::SmallVec;

use crate::na::{self, Isometry2, Point2, Vector2};
use crate::nc::bounding_volume::{HasBoundingVolume, AABB};
use crate::nc::shape::Cuboid;

use crate::application::{InputContext, InputIntent};
use crate::systems::WorldViewport;

use crate::game::{self, Animation, SpriteSheetHandle, TextureInfo};

use num::traits::Zero;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn as_vec(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }

    #[allow(unused)]
    pub fn as_pnt(self) -> Point2<f32> {
        Point2::new(self.x, self.y)
    }
}

// Although vx and vy are always 0.0 at the end of an update,
// last_pos changes, and therefore we simply transmit the whole thing.
// XXX this is probably broken right now, because we don't always transmit it when last_pos changes...
// TODO impl Serialize manually, only transmit last_pos.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,

    pub last_pos: Position,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Movement {
    pub vel: Vector2<f32>,
    pub max_vel: Vector2<f32>,
    pub acc: Vector2<f32>,
}

impl Movement {
    pub fn new(max_vel: Vector2<f32>, acc: Vector2<f32>) -> Movement {
        Movement {
            vel: Vector2::zero(),
            max_vel,
            acc,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JumpState {
    Idle,
    Rising,
    MidairIdle,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Jump {
    pub state: JumpState,
    pub jump_time_remaining: f32,
}

impl Default for Jump {
    fn default() -> Self {
        Jump::new()
    }
}

impl Jump {
    pub fn new() -> Jump {
        Jump {
            state: JumpState::Idle,
            jump_time_remaining: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gravity {
    pub f: f32,
}

impl Default for Gravity {
    fn default() -> Self {
        Gravity::new()
    }
}

impl Gravity {
    pub fn new() -> Gravity {
        Gravity { f: 1.0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionType {
    Solid,
    Trigger,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollisionShape {
    coll_type: CollisionType,
    #[serde(with = "crate::net::serde_impls::cuboid")]
    r_x: Cuboid<f32>,
    off_x: Vector2<f32>,
    #[serde(with = "crate::net::serde_impls::cuboid")]
    r_y: Cuboid<f32>,
    off_y: Vector2<f32>,
    pub ongoing_collisions: OngoingCollisions,
}

impl CollisionShape {
    pub fn new_single(
        rect: Cuboid<f32>,
        off: Vector2<f32>,
        collision_type: CollisionType,
    ) -> CollisionShape {
        Self::new_dual(rect.clone(), off, rect, off, collision_type)
    }

    pub fn new_dual(
        rect_x: Cuboid<f32>,
        off_x: Vector2<f32>,
        rect_y: Cuboid<f32>,
        off_y: Vector2<f32>,
        collision_type: CollisionType,
    ) -> CollisionShape {
        CollisionShape {
            coll_type: collision_type,
            r_x: rect_x,
            off_x,
            r_y: rect_y,
            off_y,
            ongoing_collisions: Default::default(),
        }
    }

    pub fn collision_type(&self) -> CollisionType {
        self.coll_type
    }

    // pub fn rect_x(&self) -> &Cuboid<f32> {
    //     &self.r_x
    // }

    pub fn aabb_x(&self, pos: Vector2<f32>) -> AABB<f32> {
        self.r_x
            .bounding_volume(&Isometry2::new(pos + self.off_x, na::zero()))
    }

    pub fn off_x(&self) -> &Vector2<f32> {
        &self.off_x
    }

    // pub fn rect_y(&self) -> &Cuboid<f32> {
    //     &self.r_y
    // }

    pub fn aabb_y(&self, pos: Vector2<f32>) -> AABB<f32> {
        self.r_y
            .bounding_volume(&Isometry2::new(pos + self.off_y, na::zero()))
    }

    pub fn off_y(&self) -> &Vector2<f32> {
        &self.off_y
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OngoingCollisions {
    pub others: SmallVec<[Entity; 4]>,
}

impl OngoingCollisions {
    pub fn new() -> OngoingCollisions {
        Default::default()
    }

    pub fn contains(&self, other: Entity) -> bool {
        self.others.contains(&other)
    }

    /// Adds the given entity to the set of ongoing collisions, and returns true if `other` was not already contained
    pub fn add(&mut self, other: Entity) -> bool {
        if !self.contains(other) {
            self.others.push(other);
            true
        } else {
            false
        }
    }

    /// Removes `other` from the set of ongoing collisions, and returns `true` if it was contained
    pub fn remove(&mut self, other: Entity) -> bool {
        if let Some(idx) = self.others.iter().position(|e| *e == other) {
            self.others.swap_remove(idx);
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interactor;

impl Interactor {
    pub fn can_interact(&self, _interaction: &game::Interaction) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteractionPossibility {
    pub interaction: game::Interaction,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteLayer {
    Background,
    Foreground,
}

impl SpriteLayer {
    pub const MAX_DEPTH: f32 = 1.0;

    pub fn to_depth(self) -> f32 {
        match self {
            SpriteLayer::Background => 0.1,
            SpriteLayer::Foreground => 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteInfo {
    pub width: f32,
    pub height: f32,
    pub texture_info: TextureInfo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub info: SpriteInfo,
    pub sprite_layer: SpriteLayer,
}

#[derive(Clone, Debug)]
pub struct SpriteSheetAnimation {
    pub sheet_handle: SpriteSheetHandle,
    pub animation: Animation,
    pub current_frame: u8,
    pub frame_time_remaining: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub world_viewport: WorldViewport,
    #[serde(with = "crate::net::serde_impls::aabb")]
    pub screen_viewport: AABB<f32>,
    pub resize_world_to_window: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyboardInput {
    pub input_context: InputContext,
}

pub type Intents = HashSet<InputIntent>;

impl Camera {
    pub fn new(
        world_viewport: WorldViewport,
        screen_viewport: AABB<f32>,
        resize_world_to_window: bool,
    ) -> Camera {
        Camera {
            world_viewport,
            screen_viewport,
            resize_world_to_window,
        }
    }

    // #[allow(dead_code)]
    // pub fn new_empty() -> Camera {
    //     Camera {
    //         world_viewport: WorldViewport::new_empty(),
    //         screen_viewport: AABB::new(),
    //         resize_world_to_window: true,
    //     }
    // }
}

components! {
    struct LevelComponents {
        #[hot] position: Position,
        #[hot] collision_shape: CollisionShape,
        #[hot] sprite: Sprite,
        #[cold] sprite_sheet_animation: SpriteSheetAnimation,
        #[cold] movement: Movement,
        #[cold] facing: Facing,
        #[cold] jump: Jump,
        #[cold] velocity: Velocity,
        #[cold] gravity: Gravity,
        #[cold] camera: Camera,
        #[cold] keyboard_input: KeyboardInput,
        #[cold] intents: Intents,
        #[cold] interactor: Interactor,
        #[cold] interaction_possibility: InteractionPossibility,
    }
}

#[derive(Debug, Default)]
pub struct LevelChangedFlags {
    pub position: HashMap<Entity, Position>,
    pub collision_shape: HashMap<Entity, CollisionShape>,
    pub sprite: HashMap<Entity, Sprite>,
    pub sprite_sheet_animation: HashMap<Entity, SpriteSheetAnimation>,
    pub movement: HashMap<Entity, Movement>,
    pub facing: HashMap<Entity, Facing>,
    pub jump: HashMap<Entity, Jump>,
    pub velocity: HashMap<Entity, Velocity>,
    pub gravity: HashMap<Entity, Gravity>,
    pub camera: HashMap<Entity, Camera>,
    // we don't want to transmit keyboard_input for now
    pub keyboard_input: HashMap<Entity, KeyboardInput>,
    pub intents: HashMap<Entity, Intents>,
    pub interactor: HashMap<Entity, Interactor>,
    pub interaction_possibility: HashMap<Entity, InteractionPossibility>,
}

impl LevelChangedFlags {
    pub fn clear(&mut self) {
        self.position.clear();
        self.collision_shape.clear();
        self.sprite.clear();
        self.sprite_sheet_animation.clear();
        self.movement.clear();
        self.facing.clear();
        self.jump.clear();
        self.velocity.clear();
        self.gravity.clear();
        self.camera.clear();
        self.keyboard_input.clear();
        self.intents.clear();
        self.interactor.clear();
        self.interaction_possibility.clear();
    }
}
