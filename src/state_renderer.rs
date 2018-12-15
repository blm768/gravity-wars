use std::fmt::Debug;

use cgmath::Matrix4;
use std::error::Error;
use std::rc::Rc;

use crate::rendering::context::RenderingContext;
use crate::rendering::light::PointLight;
use crate::rendering::material::BoundMaterialShader;
use crate::rendering::material::{MaterialShader, MaterialWorldContext};
use crate::rendering::mesh::Mesh;
use crate::state::{Entity, EntityRenderer, GameState};

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
    fn render(&self, entity: &Entity, world: &GameState) {
        let context = self.renderer.context();
        let mat_shader = self.renderer.material_shader();
        let bound_shader = BoundMaterialShader::new(context, mat_shader, world).unwrap();

        let model_transform = Matrix4::from_translation(entity.position);
        bound_shader.set_uniform_mat4(
            self.renderer.material_shader().info.model_view.index,
            model_transform,
        );
        self.mesh.draw(&bound_shader);
    }
}

pub trait GameRenderer: Debug {
    type Context: RenderingContext;

    fn context(&self) -> &Self::Context;
    fn material_shader(&self) -> &MaterialShader<Self::Context>;
    fn render(&self, state: &mut GameState) -> Result<(), Box<Error>>;
}

impl MaterialWorldContext for GameState {
    fn projection(&self) -> Matrix4<f32> {
        self.camera.projection().into()
    }

    fn light(&self) -> &PointLight {
        &self.light
    }
}
