use std::collections::HashMap;
use std::fmt::Debug;

use cgmath::{Matrix4, Vector3, Vector4};

#[derive(Clone, Debug)]
pub struct ShaderParamInfo {
    pub index: usize,
}

pub trait ShaderProgram: Debug {
    fn attributes(&self) -> HashMap<Box<str>, ShaderParamInfo>;
    fn uniforms(&self) -> HashMap<Box<str>, ShaderParamInfo>;

    fn activate(&self);
    fn set_uniform_f32(&self, index: usize, value: f32);
    fn set_uniform_mat4(&self, index: usize, value: Matrix4<f32>);
    fn set_uniform_vec3(&self, index: usize, value: Vector3<f32>);
    fn set_uniform_vec4(&self, index: usize, value: Vector4<f32>);
}
