use rendering::buffer;
use rendering::mesh::ElementIndices;

pub trait RenderingContext {
    type AttributeBuffer: buffer::AttributeBuffer<RenderingContext = Self> + 'static;
    type IndexBuffer: buffer::IndexBuffer<RenderingContext = Self> + 'static;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()>;
    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()>;

    fn draw_triangles(&self, count: usize);
    fn draw_indexed_triangles(&self, indices: &ElementIndices<Self>);
}
