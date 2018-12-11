use std::fmt::Debug;

use cgmath::Matrix4;
use std::error::Error;
use std::rc::Rc;

use rendering::context::RenderingContext;
use rendering::material::MaterialShader;
use rendering::mesh::Mesh;
use rendering::shader::ShaderProgram;
use state::{Entity, EntityRenderer, GameState};

#[derive(Debug)]
pub struct MeshRenderer<Context: RenderingContext> {
    mesh: Mesh<Context>,
    renderer: Rc<GameRenderer<Context = Context>>,
}

impl<Context: RenderingContext> MeshRenderer<Context> {
    pub fn new(renderer: Rc<GameRenderer<Context = Context>>, mesh: Mesh<Context>) -> Self {
        MeshRenderer { mesh, renderer }
    }
}

impl<Context: RenderingContext> EntityRenderer for MeshRenderer<Context> {
    fn render(&self, entity: &Entity) {
        let model_transform = Matrix4::from_translation(entity.position);
        self.renderer.material_shader().program.set_uniform_mat4(
            self.renderer.material_shader().info.model_view.index,
            model_transform,
        );
        self.mesh
            .draw(self.renderer.context(), self.renderer.material_shader());
    }
}

pub trait GameRenderer: Debug {
    type Context: RenderingContext;

    fn context(&self) -> &Self::Context;
    fn material_shader(&self) -> &MaterialShader<Self::Context>;
    fn render(&self, state: &mut GameState) -> Result<(), Box<Error>>;
}
