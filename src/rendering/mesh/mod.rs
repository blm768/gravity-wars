use std::rc::Rc;

use rendering::buffer::{AttributeBuffer, ElementBinding, VertexAttributeBinding};
use rendering::context::RenderingContext;
use rendering::material::Material;
use rendering::shader::{MaterialShaderInfo, ShaderProgram};

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

    pub fn draw(&self, context: &Context, program: &ShaderProgram, info: &MaterialShaderInfo) {
        for p in self.primitives.iter() {
            p.draw(context, program, info);
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
    pub fn draw(&self, context: &Context, program: &ShaderProgram, info: &MaterialShaderInfo) {
        self.material.set_uniforms(program, info);
        self.positions.bind(info.position.index);
        self.normals.bind(info.normal.index);
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
}
