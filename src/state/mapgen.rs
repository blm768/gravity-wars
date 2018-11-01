use specs::Builder;

use state::{GameState, Position};

pub fn generate_map(state: &mut GameState) {
    let world = &mut state.world;
    world.create_entity().with(Position::new(0.0, 0.0)).build();
}
