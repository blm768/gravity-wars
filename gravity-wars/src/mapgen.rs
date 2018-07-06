use cgmath::Vector2;

use state::{Entity, Map};

pub fn generate_map() -> Map {
    let mut map = Map::new();
    map.entities.push(Entity::new(Vector2::new(0.0, 0.0)));
    map
}
