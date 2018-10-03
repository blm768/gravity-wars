pub mod light;
pub mod material;
pub mod mesh;
pub mod renderer;
pub mod scene;
pub mod shader;

use cgmath::{Vector3, Vector4};
use rgb::{RGB, RGBA};

pub type Rgb = RGB<f32>;
pub type Rgba = RGBA<f32>;

fn rgb_as_vec3(rgb: &Rgb) -> Vector3<f32> {
    Vector3::new(rgb.r, rgb.b, rgb.g)
}

fn rgba_as_vec4(rgba: &Rgba) -> Vector4<f32> {
    Vector4::new(rgba.r, rgba.b, rgba.g, rgba.a)
}
