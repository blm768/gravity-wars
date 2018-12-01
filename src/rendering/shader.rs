use std::fmt::Debug;

use cgmath::{Matrix4, Vector3, Vector4};

use crate::rendering::context::RenderingContext;
use crate::rendering::mesh::ElementIndices;

#[derive(Clone, Debug)]
pub enum ShaderInfoError {
    MissingAttribute(String),
    MissingUniform(String),
}

#[derive(Clone, Debug)]
pub struct ShaderParamInfo {
    pub index: usize,
}

impl ShaderParamInfo {
    pub fn attribute<Context: RenderingContext>(
        shader: &ShaderProgram<RenderingContext = Context>,
        name: &str,
    ) -> Result<ShaderParamInfo, ShaderInfoError> {
        shader
            .attribute(name)
            .ok_or_else(|| ShaderInfoError::MissingAttribute(String::from(name)))
    }

    pub fn uniform<Context: RenderingContext>(
        shader: &ShaderProgram<RenderingContext = Context>,
        name: &str,
    ) -> Result<ShaderParamInfo, ShaderInfoError> {
        shader
            .uniform(name)
            .ok_or_else(|| ShaderInfoError::MissingUniform(String::from(name)))
    }
}

pub trait ShaderProgram: Debug {
    type RenderingContext: RenderingContext + ?Sized;

    fn attribute_names(&self) -> Vec<String>;
    fn uniform_names(&self) -> Vec<String>;
    fn attribute(&self, name: &str) -> Option<ShaderParamInfo>;
    fn uniform(&self, name: &str) -> Option<ShaderParamInfo>;
}

pub trait BoundShader<Context: RenderingContext + ?Sized> {
    fn draw_triangles(&self, count: usize);
    fn draw_indexed_triangles(&self, indices: &ElementIndices<Context>);
    fn draw_polyline(&self, num_vertices: usize);

    fn set_uniform_f32(&self, index: usize, value: f32);
    fn set_uniform_mat4(&self, index: usize, value: Matrix4<f32>);
    fn set_uniform_vec3(&self, index: usize, value: Vector3<f32>);
    fn set_uniform_vec4(&self, index: usize, value: Vector4<f32>);
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderBindError {
    InvalidContextForShader,
    CannotBindMoreShaders,
}
