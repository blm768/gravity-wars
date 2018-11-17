use std::fmt::Debug;
use std::rc::Rc;

use cgmath::Vector3;

use rendering::light::PointLight;
use rendering::scene::Camera;
use rendering::Rgb;
use state::event::InputEvent;

pub mod event;
pub mod mapgen;

#[derive(Debug)]
pub struct Entity {
    pub position: Vector3<f32>,
    pub renderer: Option<Rc<EntityRenderer>>,
}

impl Entity {
    pub fn new(position: Vector3<f32>) -> Entity {
        Entity {
            position,
            renderer: None,
        }
    }
}

pub trait EntityRenderer: Debug {
    fn render(&self, entity: &Entity);
}

pub struct GameState {
    pub entities: Vec<Entity>,
    pub camera: Camera,
    pub light: PointLight,
}

impl GameState {
    pub fn new() -> GameState {
        let light = PointLight {
            color: Rgb::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, -3.0),
        };

        GameState {
            entities: Vec::new(),
            camera: Camera::new(),
            light,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn handle_input(&mut self, event: &InputEvent) {
        // TODO: clamp movement/scale.
        match event {
            InputEvent::PanCamera(delta) => {
                self.camera.position += Vector3::new(delta.x, delta.y, 0.0);
            }
            InputEvent::ZoomCamera(ref scale) => {
                self.camera.log_scale += scale;
            }
        }
    }
}
