use rendering;
use rendering::light::{PointLight, ShaderLightInfo};
use rendering::shader::{ShaderParamInfo, ShaderProgram};
use rendering::Rgba;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Rgba,
    pub metal_factor: f32,
    pub roughness: f32,
}

#[derive(Debug)]
pub enum ShaderInfoError {
    MissingAttribute(String),
    MissingUniform(String),
}

#[derive(Debug)]
pub struct MaterialShaderInfo {
    pub position: ShaderParamInfo,
    pub normal: ShaderParamInfo,

    pub projection: ShaderParamInfo,
    pub model_view: ShaderParamInfo,
    pub base_color: Option<ShaderParamInfo>,
    pub metal_factor: Option<ShaderParamInfo>,
    pub roughness: Option<ShaderParamInfo>,

    pub lights: Option<ShaderLightInfo>,
}

impl MaterialShaderInfo {
    pub fn bind_material(&self, material: &Material, program: &ShaderProgram) {
        if let Some(ref base_color) = self.base_color {
            program.set_uniform_vec4(
                base_color.index,
                rendering::rgba_as_vec4(&material.base_color),
            );
        }
        if let Some(ref metal_factor) = self.metal_factor {
            program.set_uniform_f32(metal_factor.index, material.metal_factor);
        }
        if let Some(ref roughness) = self.roughness {
            program.set_uniform_f32(roughness.index, material.roughness);
        }
    }
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
            normal: get_attribute("normal")?,
            projection: get_uniform("projection")?,
            model_view: get_uniform("modelView")?,
            base_color: get_uniform("material.baseColor").ok(),
            metal_factor: get_uniform("material.metal").ok(),
            roughness: get_uniform("material.roughness").ok(),
            lights: ShaderLightInfo::from_program(program),
        })
    }
}

#[derive(Debug)]
pub struct MaterialShader {
    pub program: Box<ShaderProgram>,
    pub info: MaterialShaderInfo,
}

impl MaterialShader {
    pub fn new(program: Box<ShaderProgram>) -> Result<MaterialShader, ShaderInfoError> {
        let info = MaterialShaderInfo::from_program(&*program)?;
        Ok(MaterialShader { program, info })
    }

    pub fn bind_material(&self, material: &Material) {
        self.info.bind_material(material, &*self.program);
    }

    pub fn bind_light(&self, light: &PointLight) {
        if let Some(ref light_info) = self.info.lights {
            light_info.bind_light(light, &*self.program);
        }
    }
}
