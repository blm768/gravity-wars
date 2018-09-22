use std::collections::HashMap;

use cgmath::{Matrix4, Vector4};

#[derive(Clone, Debug)]
pub struct ShaderParamInfo {
    pub index: usize,
}

pub trait ShaderProgram {
    fn attributes(&self) -> HashMap<Box<str>, ShaderParamInfo>;
    fn uniforms(&self) -> HashMap<Box<str>, ShaderParamInfo>;

    fn activate(&self);
    fn set_uniform_mat4(&self, index: usize, value: Matrix4<f32>);
    fn set_uniform_vec4(&self, index: usize, value: Vector4<f32>);
}

#[derive(Debug)]
pub struct MaterialShaderInfo {
    pub position: ShaderParamInfo,
    pub projection: ShaderParamInfo,
    pub model_view: ShaderParamInfo,
    pub base_color: Option<ShaderParamInfo>,
}

#[derive(Debug)]
pub enum ShaderInfoError {
    MissingAttribute(String),
    MissingUniform(String),
}

impl MaterialShaderInfo {
    pub fn from_program(program: &ShaderProgram) -> Result<MaterialShaderInfo, ShaderInfoError> {
        let attributes = program.attributes();
        let uniforms = program.uniforms();
        let get_attribute = |name: &str| match attributes.get(name) {
            Some(info) => Ok(info.clone()),
            None => Err(ShaderInfoError::MissingAttribute(String::from(name))),
        };
        let get_uniform = |name: &str| match uniforms.get(name) {
            Some(info) => Ok(info.clone()),
            None => Err(ShaderInfoError::MissingUniform(String::from(name))),
        };

        Ok(MaterialShaderInfo {
            position: get_attribute("position")?,
            projection: get_uniform("projection")?,
            model_view: get_uniform("modelView")?,
            base_color: get_uniform("baseColor").ok(),
        })
    }
}
