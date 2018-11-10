use std::rc::Rc;

use cgmath::Vector2;

use state::{Entity, EntityRenderer, GameState};

pub fn generate_map(state: &mut GameState) {}

pub fn add_ships(state: &mut GameState, ship_renderer: Rc<EntityRenderer>) {
    let mut ship = Entity::new(Vector2::new(0.0, 0.0));
    ship.renderer = Some(ship_renderer);
    state.entities.push(ship)
}
