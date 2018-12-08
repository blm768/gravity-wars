use std::fmt::Debug;

use rendering::buffer;
use rendering::mesh::ElementIndices;
use rendering::shader;

pub trait RenderingContext: Debug {
    type AttributeBuffer: buffer::AttributeBuffer<RenderingContext = Self> + 'static;
    type IndexBuffer: buffer::IndexBuffer<RenderingContext = Self> + 'static;
    type ShaderProgram: shader::ShaderProgram<RenderingContext = Self> + 'static;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()>;
    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()>;

    fn draw_triangles(&self, count: usize);
    fn draw_indexed_triangles(&self, indices: &ElementIndices<Self>);
}
