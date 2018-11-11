use std::rc::Rc;

use cgmath::Vector3;

use state::{Entity, EntityRenderer, GameState};

pub fn generate_map(state: &mut GameState) {}

pub fn add_ships(state: &mut GameState, ship_renderer: Rc<EntityRenderer>) {
    let mut ship = Entity::new(Vector3::new(0.0, 0.0, 0.0));
    ship.renderer = Some(ship_renderer);
    state.entities.push(ship)
}
