use std::error::Error;

use state::GameState;

pub trait GameRenderer {
    fn render(&self, &GameState) -> Result<(), Box<Error>>;
}
