use std::fmt::Debug;
use std::rc::Rc;

use rendering::buffer;
use rendering::shader;
use rendering::shader::{BoundShader, ShaderBindError};

pub trait RenderingContext: Debug {
    type AttributeBuffer: buffer::AttributeBuffer<RenderingContext = Self> + 'static;
    type IndexBuffer: buffer::IndexBuffer<RenderingContext = Self> + 'static;
    type ShaderProgram: shader::ShaderProgram<RenderingContext = Self> + 'static;
    type BoundShader: BoundShader<Self> + 'static;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()>;
    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()>;

    fn bind_shader(
        &self,
        shader: Rc<Self::ShaderProgram>,
    ) -> Result<Self::BoundShader, ShaderBindError>;
}
