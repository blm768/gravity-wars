use cgmath::Vector3;

use rendering;
use rendering::shader::{ShaderLightInfo, ShaderProgram};
use rendering::Rgb;

pub struct PointLight {
    pub color: Rgb,
    pub position: Vector3<f32>,
}

impl PointLight {
    pub fn bind(&self, program: &ShaderProgram, info: &ShaderLightInfo) {
        program.set_uniform_vec3(info.color.index, rendering::rgb_as_vec3(&self.color));
        program.set_uniform_vec3(info.position.index, self.position);
    }
}
