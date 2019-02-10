use std::fmt::Debug;
use std::rc::Rc;

use crate::rendering::buffer;
use crate::rendering::shader;
use crate::rendering::shader::{BoundShader, ShaderBindError};
use crate::rendering::texture::Texture;

pub trait RenderingContext: Debug {
    type AttributeBuffer: buffer::AttributeBuffer<RenderingContext = Self> + 'static;
    type IndexBuffer: buffer::IndexBuffer<RenderingContext = Self> + 'static;
    type ShaderProgram: shader::ShaderProgram<RenderingContext = Self> + 'static;
    type BoundShader: BoundShader<Self> + 'static;
    type Texture: Texture<RenderingContext = Self> + 'static;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()>;
    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()>;
    fn make_texture(&self) -> Result<Self::Texture, ()>;

    fn bind_shader(
        &self,
        shader: Rc<Self::ShaderProgram>,
    ) -> Result<Self::BoundShader, ShaderBindError>;
}
