use cgmath::Vector2;

use rendering::scene::Camera;

pub struct Entity {
    pub position: Vector2<f32>,
}

impl Entity {
    pub fn new(position: Vector2<f32>) -> Entity {
        Entity { position }
    }
}

pub struct Map {
    pub entities: Vec<Entity>,
}

impl Map {
    pub fn new() -> Map {
        Map { entities: vec![] }
    }
}

pub struct GameState {
    pub camera: Camera,
    pub map: Map,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            camera: Camera::new(),
            map: Map::new(),
        }
    }
}
