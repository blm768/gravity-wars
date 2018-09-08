use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ShaderParamInfo {
    pub index: usize,
}

pub trait ShaderProgram {
    fn attributes(&self) -> HashMap<Box<str>, ShaderParamInfo>;
    fn uniforms(&self) -> HashMap<Box<str>, ShaderParamInfo>;

    fn activate(&self);
}

#[derive(Debug)]
pub struct MaterialShaderInfo {
    pub position: ShaderParamInfo,
    pub projection: ShaderParamInfo,
    pub model_view: ShaderParamInfo,
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

        let position = get_attribute("position")?;
        let projection = get_uniform("projection")?;
        let model_view = get_uniform("modelView")?;

        Ok(MaterialShaderInfo {
            position,
            projection,
            model_view,
        })
    }
}
