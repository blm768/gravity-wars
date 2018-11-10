use std::fmt::Debug;
use std::rc::Rc;

use cgmath::Vector2;

use rendering::scene::Camera;

pub mod mapgen;

#[derive(Debug)]
pub struct Entity {
    pub position: Vector2<f32>,
    pub renderer: Option<Rc<EntityRenderer>>,
}

impl Entity {
    pub fn new(position: Vector2<f32>) -> Entity {
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
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            entities: Vec::new(),
            camera: Camera::new(),
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
