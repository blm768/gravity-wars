use cgmath::Vector2;

use state::{Entity, GameState};

pub fn generate_map(state: &mut GameState) {}

pub fn add_ships(state: &mut GameState) {
    let ship = Entity::new(Vector2::new(0.0, 0.0));
    state.entities.push(ship)
}
