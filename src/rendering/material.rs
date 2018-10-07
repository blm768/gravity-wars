use rendering;
use rendering::shader::{MaterialShaderInfo, ShaderProgram};
use rendering::Rgba;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Rgba,
    pub metal_factor: f32,
    pub roughness: f32,
}

impl Material {
    pub fn set_uniforms(&self, program: &ShaderProgram, info: &MaterialShaderInfo) {
        if let Some(ref base_color) = info.base_color {
            program.set_uniform_vec4(base_color.index, rendering::rgba_as_vec4(&self.base_color));
        }
        if let Some(ref metal_factor) = info.metal_factor {
            program.set_uniform_f32(metal_factor.index, self.metal_factor);
        }
        if let Some(ref roughness) = info.roughness {
            program.set_uniform_f32(roughness.index, self.roughness);
        }
    }
}
