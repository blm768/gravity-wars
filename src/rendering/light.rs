use cgmath::Vector3;

use rendering;
use rendering::shader::{ShaderParamInfo, ShaderProgram};
use rendering::Rgb;

pub struct PointLight {
    pub color: Rgb,
    pub position: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct ShaderLightInfo {
    pub color: ShaderParamInfo,
    pub position: ShaderParamInfo,
}

impl ShaderLightInfo {
    pub fn from_program(program: &ShaderProgram) -> Option<ShaderLightInfo> {
        let uniforms = program.uniforms();
        Some(ShaderLightInfo {
            color: uniforms.get("light.color")?.clone(),
            position: uniforms.get("light.position")?.clone(),
        })
    }

    pub fn bind_light(&self, light: &PointLight, program: &ShaderProgram) {
        program.set_uniform_vec3(self.color.index, rendering::rgb_as_vec3(&light.color));
        program.set_uniform_vec3(self.position.index, light.position);
    }
}
