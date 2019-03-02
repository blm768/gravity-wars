use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use crate::rendering::buffer;
use crate::rendering::shader;
use crate::rendering::shader::{BoundShader, ShaderBindError, ShaderType};
use crate::rendering::texture::Texture;

pub trait RenderingContext: Debug {
    type AttributeBuffer: buffer::AttributeBuffer<RenderingContext = Self> + 'static;
    type IndexBuffer: buffer::IndexBuffer<RenderingContext = Self> + 'static;
    type Shader: shader::Shader<RenderingContext = Self> + 'static;
    type ShaderCreationError: Error;
    type ShaderProgram: shader::ShaderProgram<RenderingContext = Self> + 'static;
    type ShaderLinkError: Error;
    type BoundShader: BoundShader<Self> + 'static;
    type Texture: Texture<RenderingContext = Self> + 'static;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()>;
    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()>;
    fn make_texture(&self) -> Result<Self::Texture, ()>;
    fn compile_shader(
        &self,
        shader_type: ShaderType,
        source: &str,
    ) -> Result<Self::Shader, Self::ShaderCreationError>;
    fn link_shader_program<'a, T: Iterator<Item = &'a Self::Shader>>(
        &self,
        shaders: T,
    ) -> Result<Self::ShaderProgram, Self::ShaderLinkError>;

    fn bind_shader(
        &self,
        shader: Rc<Self::ShaderProgram>,
    ) -> Result<Self::BoundShader, ShaderBindError>;
}
