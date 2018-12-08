use std::rc::Rc;

use rendering::buffer::{AttributeBuffer, Buffer, BufferData, ElementBinding};
use rendering::buffer::{VertexAttributeBinding, VertexIndexData};
use rendering::context::RenderingContext;
use rendering::material::{Material, MaterialShader};

pub mod gltf;

#[derive(Debug)]
pub struct Mesh<Context: RenderingContext> {
    primitives: Vec<Primitive<Context>>,
}

impl<Context: RenderingContext> Mesh<Context> {
    pub fn new(primitives: Vec<Primitive<Context>>) -> Self {
        Mesh { primitives }
    }

    pub fn primitives(&self) -> &[Primitive<Context>] {
        &self.primitives
    }

    pub fn draw(&self, context: &Context, mat_shader: &MaterialShader<Context>) {
        for p in self.primitives.iter() {
            p.draw(context, mat_shader);
        }
    }
}

#[derive(Debug)]
pub struct Primitive<Context: RenderingContext> {
    material: Material,
    indices: Option<ElementIndices<Context>>,
    positions: VertexAttribute<Context>,
    normals: VertexAttribute<Context>,
}

impl<Context: RenderingContext> Primitive<Context> {
    pub fn new(
        material: Material,
        indices: Option<ElementIndices<Context>>,
        positions: VertexAttribute<Context>,
        normals: VertexAttribute<Context>,
    ) -> Result<Self, ()> {
        Ok(Primitive {
            material,
            indices,
            positions,
            normals,
        })
    }

    /// Binds each primitive's buffers and makes the appropriate WebGL draw calls.
    /// The projection and modelview matrix uniforms must already be bound.
    // TODO: break out a separate bind() method?
    pub fn draw(&self, context: &Context, mat_shader: &MaterialShader<Context>) {
        mat_shader.bind_material(&self.material);
        self.positions.bind(mat_shader.info.position.index);
        self.normals.bind(mat_shader.info.normal.index);
        match self.indices {
            Some(ref indices) => context.draw_indexed_triangles(indices),
            None => context.draw_triangles(self.positions.binding.count),
        }
    }
}

#[derive(Debug)]
pub struct VertexAttribute<Context: RenderingContext> {
    buffer: Rc<Context::AttributeBuffer>,
    binding: VertexAttributeBinding,
}

impl<Context: RenderingContext> VertexAttribute<Context> {
    pub fn new(buffer: Rc<Context::AttributeBuffer>, binding: VertexAttributeBinding) -> Self {
        VertexAttribute { buffer, binding }
    }

    /// Binds the primitive's buffers and makes the appropriate WebGL draw call.
    /// The projection and modelview matrix uniforms must already be bound.
    pub fn bind(&self, index: usize) {
        self.buffer.bind_to_attribute(index, &self.binding);
    }
}

#[derive(Debug)]
pub struct ElementIndices<Context: RenderingContext + ?Sized> {
    pub buffer: Rc<Context::IndexBuffer>,
    pub binding: ElementBinding,
}

impl<Context: RenderingContext> ElementIndices<Context> {
    pub fn new(buffer: Rc<Context::IndexBuffer>, binding: ElementBinding) -> Self {
        ElementIndices { buffer, binding }
    }

    pub fn from_data<E>(
        data: &BufferData<E>,
        context: &Context,
    ) -> Result<ElementIndices<Context>, ()>
    where
        E: VertexIndexData,
    {
        let index_buf = context.make_index_buffer()?;
        index_buf.set_data(data.as_bytes());
        Ok(ElementIndices::new(
            Rc::new(index_buf),
            ElementBinding {
                index_type: E::INDEX_TYPE,
                count: data.num_elements(),
                offset: 0,
            },
        ))
    }
}
