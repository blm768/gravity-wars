use rendering;
use rendering::context::RenderingContext;
use rendering::light::{PointLight, ShaderLightInfo};
use rendering::shader::{ShaderInfoError, ShaderParamInfo, ShaderProgram};
use rendering::Rgba;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Rgba,
    pub metal_factor: f32,
    pub roughness: f32,
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
    pub fn bind_material<Context: RenderingContext>(
        &self,
        material: &Material,
        program: &ShaderProgram<RenderingContext = Context>,
    ) {
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
    pub fn from_program<Context: RenderingContext>(
        program: &ShaderProgram<RenderingContext = Context>,
    ) -> Result<MaterialShaderInfo, ShaderInfoError> {
        Ok(MaterialShaderInfo {
            position: ShaderParamInfo::attribute(program, "position")?,
            normal: ShaderParamInfo::attribute(program, "normal")?,
            projection: ShaderParamInfo::uniform(program, "projection")?,
            model_view: ShaderParamInfo::uniform(program, "modelView")?,
            base_color: ShaderParamInfo::uniform(program, "material.baseColor").ok(),
            metal_factor: ShaderParamInfo::uniform(program, "material.metal").ok(),
            roughness: ShaderParamInfo::uniform(program, "material.roughness").ok(),
            lights: ShaderLightInfo::from_program(program),
        })
    }
}

#[derive(Debug)]
pub struct MaterialShader<Context: RenderingContext> {
    pub program: Context::ShaderProgram,
    pub info: MaterialShaderInfo,
}

impl<Context: RenderingContext> MaterialShader<Context> {
    pub fn new(program: Context::ShaderProgram) -> Result<Self, ShaderInfoError> {
        let info = MaterialShaderInfo::from_program(&program)?;
        Ok(MaterialShader { program, info })
    }

    pub fn bind_material(&self, material: &Material) {
        self.info.bind_material(material, &self.program);
    }

    pub fn bind_light(&self, light: &PointLight) {
        if let Some(ref light_info) = self.info.lights {
            light_info.bind_light(light, &self.program);
        }
    }
}
