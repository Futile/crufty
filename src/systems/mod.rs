pub use self::render_system::{ RenderSystem, WorldViewport };
pub use self::camera_system::{ CameraSystem };
pub use self::keyboard_system::KeyboardSystem;
pub use self::intent_system::IntentSystem;

use ecs::system::{ LazySystem, EntitySystem, InteractSystem };

use components::LevelComponents;

use util::TextureStore;

mod render_system;
mod camera_system;
mod keyboard_system;
mod intent_system;

services! {
    struct LevelServices {
        texture_store: TextureStore = TextureStore::new_invalid(),
    }
}

systems! {
    struct LevelSystems<LevelComponents, LevelServices> {
        render_system: LazySystem<InteractSystem<RenderSystem>> = LazySystem::new(),
        camera_system: EntitySystem<CameraSystem> = EntitySystem::new(
            CameraSystem::new(),
            aspect!(<LevelComponents> all: [camera])),
        keyboard_system: EntitySystem<KeyboardSystem> = EntitySystem::new(
            KeyboardSystem::new(),
            aspect!(<LevelComponents> all: [keyboard_input])),
        intent_system: EntitySystem<IntentSystem> = EntitySystem::new(
            IntentSystem,
            aspect!(<LevelComponents> all: [intents])),
    }
}

