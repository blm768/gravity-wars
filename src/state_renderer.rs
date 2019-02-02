use std::cell::RefCell;
use std::fmt::Debug;

use nalgebra::Matrix4;
use std::cell::Cell;
use std::error::Error;
use std::rc::Rc;

use crate::rendering::context::RenderingContext;
use crate::rendering::light::SunLight;
use crate::rendering::line::{BoundLineShader, LineShader, LineWorldContext, PolyLine};
use crate::rendering::material::{BoundMaterialShader, MaterialShader, MaterialWorldContext};
use crate::rendering::mesh::Mesh;
use crate::rendering::Rgb;
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

        let model_transform = entity.transform.to_similarity().to_homogeneous();
        bound_shader.set_uniform_mat4(
            self.renderer.material_shader().info.model_transform.index,
            model_transform,
        );
        self.mesh.draw(&bound_shader);
    }
}

#[derive(Debug)]
pub struct MissileTrailRenderer<Context: RenderingContext> {
    line: RefCell<PolyLine<Context>>,
    data_version: Cell<usize>,
    renderer: Rc<GameRenderer<Context = Context>>,
}

impl<Context: RenderingContext> MissileTrailRenderer<Context> {
    pub fn new(renderer: Rc<GameRenderer<Context = Context>>, color: Rgb) -> Result<Self, ()> {
        let line = PolyLine::new(renderer.context().make_attribute_buffer()?, color);
        Ok(MissileTrailRenderer {
            line: RefCell::new(line),
            data_version: Cell::new(0),
            renderer,
        })
    }
}

impl<Context: RenderingContext> EntityRenderer for MissileTrailRenderer<Context> {
    fn render(&self, entity: &Entity, world: &GameState) {
        if let Some(ref trail) = entity.missile_trail {
            if trail.data_version() != self.data_version.get() {
                self.line.borrow_mut().set_positions(trail.positions());
                self.data_version.set(trail.data_version());
            }

            let context = self.renderer.context();
            let line_shader = self.renderer.line_shader();
            let bound_shader = BoundLineShader::new(context, line_shader, world).unwrap();

            self.line.borrow().draw(&bound_shader);
        }
    }
}

pub trait GameRenderer: Debug {
    type Context: RenderingContext;

    fn context(&self) -> &Self::Context;
    fn material_shader(&self) -> &MaterialShader<Self::Context>;
    fn line_shader(&self) -> &LineShader<Self::Context>;
    fn render(&self, state: &mut GameState) -> Result<(), Box<Error>>;
}

impl MaterialWorldContext for GameState {
    fn projection(&self) -> Matrix4<f32> {
        self.camera.projection().into()
    }

    fn view(&self) -> Matrix4<f32> {
        self.camera.view().into()
    }

    fn sun(&self) -> &SunLight {
        &self.light.sun
    }

    fn ambient(&self) -> Rgb {
        self.light.ambient
    }
}

impl LineWorldContext for GameState {
    fn projection(&self) -> Matrix4<f32> {
        self.camera.projection().into()
    }

    fn view(&self) -> Matrix4<f32> {
        self.camera.view().into()
    }
}
