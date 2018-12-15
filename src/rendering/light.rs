use cgmath::Vector3;

use crate::rendering;
use crate::rendering::context::RenderingContext;
use crate::rendering::shader::{BoundShader, ShaderParamInfo, ShaderProgram};
use crate::rendering::Rgb;

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
    pub fn from_program<Context: RenderingContext>(
        program: &ShaderProgram<RenderingContext = Context>,
    ) -> Option<ShaderLightInfo> {
        Some(ShaderLightInfo {
            color: program.uniform("light.color")?,
            position: program.uniform("light.position")?,
        })
    }

    pub fn bind_light<Context: RenderingContext>(
        &self,
        light: &PointLight,
        shader: &BoundShader<Context>,
    ) {
        shader.set_uniform_vec3(self.color.index, rendering::rgb_as_vec3(&light.color));
        shader.set_uniform_vec3(self.position.index, light.position);
    }
}
