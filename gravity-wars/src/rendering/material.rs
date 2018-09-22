use cgmath::Vector4;

use rendering::shader::{MaterialShaderInfo, ShaderProgram};
use rendering::Rgba;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Rgba,
}

impl Material {
    pub fn set_uniforms(&self, program: &ShaderProgram, info: &MaterialShaderInfo) {
        if let Some(ref base_color) = info.base_color {
            program.set_uniform_vec4(base_color.index, rgba_as_vec4(&self.base_color));
        }
    }
}

fn rgba_as_vec4(rgba: &Rgba) -> Vector4<f32> {
    Vector4::new(rgba.r, rgba.b, rgba.g, rgba.a)
}
