use std::rc::Rc;

use crate::rendering::buffer::{AttributeBuffer, Buffer, BufferData, ElementBinding};
use crate::rendering::buffer::{VertexAttributeBinding, VertexIndexData};
use crate::rendering::context::RenderingContext;
use crate::rendering::material::{BoundMaterialShader, Material};

pub mod gltf;

#[derive(Debug)]
pub struct Mesh<Context: RenderingContext> {
    pub primitives: Vec<Primitive<Context>>,
    pub extras: Option<serde_json::Value>,
}

impl<Context: RenderingContext> Mesh<Context> {
    pub fn new(primitives: Vec<Primitive<Context>>) -> Self {
        Mesh { primitives, extras: None }
    }

    pub fn draw(&self, shader: &BoundMaterialShader<Context>) {
        for p in self.primitives.iter() {
            p.draw(shader);
        }
    }
}

impl<Context: RenderingContext> Clone for Mesh<Context> {
    fn clone(&self) -> Self {
        Mesh::new(self.primitives.clone())
    }
}

#[derive(Debug)]
pub struct Primitive<Context: RenderingContext> {
    pub material: Material<Context>,
    pub geometry: Rc<PrimitiveGeometry<Context>>,
}

impl<Context: RenderingContext> Primitive<Context> {
    pub fn draw(&self, shader: &BoundMaterialShader<Context>) {
        use std::ops::Deref; // TODO: why do we need to call deref() instead of using &*shader?
        shader.info().bind_material(&self.material, shader.deref());
        self.geometry.draw(shader);
    }
}

impl<Context: RenderingContext> Clone for Primitive<Context> {
    fn clone(&self) -> Self {
        Primitive {
            material: self.material.clone(),
            geometry: Rc::clone(&self.geometry),
        }
    }
}

#[derive(Debug)]
pub struct PrimitiveGeometry<Context: RenderingContext> {
    indices: Option<ElementIndices<Context>>,
    positions: VertexAttribute<Context>,
    normals: VertexAttribute<Context>,
}

impl<Context: RenderingContext> PrimitiveGeometry<Context> {
    pub fn new(
        indices: Option<ElementIndices<Context>>,
        positions: VertexAttribute<Context>,
        normals: VertexAttribute<Context>,
    ) -> Self {
        PrimitiveGeometry {
            indices,
            positions,
            normals,
        }
    }

    /// Binds each primitive's buffers and makes the appropriate WebGL draw calls.
    /// The projection and modelview matrix uniforms must already be bound.
    // TODO: break out a separate bind() method?
    pub fn draw(&self, shader: &BoundMaterialShader<Context>) {
        self.positions.bind(shader.info().position.index);
        self.normals.bind(shader.info().normal.index);
        match self.indices {
            Some(ref indices) => shader.draw_indexed_triangles(indices),
            None => shader.draw_triangles(self.positions.binding.count),
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
