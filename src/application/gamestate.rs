use std::thread;

use glium::{self, Surface};
use glutin::{self, ElementState, VirtualKeyCode};

use ecs::{World, BuildData};

use util::{State};
use application::AppTransition;

use systems::{LevelSystems, RenderSystem};
use components::{LevelComponents, Position};

pub struct GameState {
    display: glium::Display,
}

impl GameState {
    pub fn new(display: glium::Display) -> GameState {
        GameState{
            display: display,
        }
    }
}

impl State<AppTransition> for GameState {
    fn run(self: Box<Self>) -> AppTransition {
        loop {
            let mut world = World::<LevelSystems>::new();

            world.systems.render_system.init(RenderSystem);

            let _ = world.create_entity(
                |entity: BuildData<LevelComponents>, data: &mut LevelComponents| {
                    data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                }
                );

            world.update();

            for event in self.display.poll_events() {
                let mut target = self.display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                target.finish().unwrap();

                match event {
                    glutin::Event::Closed |
                    glutin::Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape))
                        => return AppTransition::Shutdown,
                    _ => ()
                }

                thread::sleep_ms(17);
            }
        }
    }
}
