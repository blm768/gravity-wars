use std::error::Error;
use std::rc::Rc;

use rendering::context::RenderingContext;
use rendering::mesh::Mesh;
use state::GameState;

#[derive(Debug)]
pub struct MeshRenderer<Context: RenderingContext> {
    mesh: Mesh<Context>,
    context: Rc<Context>,
}

impl<Context: RenderingContext> MeshRenderer<Context> {
    pub fn new(context: Rc<Context>, mesh: Mesh<Context>) -> Self {
        MeshRenderer { mesh, context }
    }
}

pub trait GameRenderer {
    fn render(&self, state: &GameState) -> Result<(), Box<Error>>;
}
