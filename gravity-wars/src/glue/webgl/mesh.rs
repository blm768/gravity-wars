use web_sys::WebGlRenderingContext;

use glue::webgl::buffer::{Buffer, ElementBinding, VertexAttributeBinding};
use glue::webgl::ShaderProgram;
use rendering::material::Material;
use rendering::shader::MaterialShaderInfo;

#[derive(Debug)]
pub struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn new(primitives: Vec<Primitive>) -> Mesh {
        Mesh { primitives }
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn draw(&self, program: &ShaderProgram, info: &MaterialShaderInfo) {
        for p in self.primitives.iter() {
            p.draw(program, info);
        }
    }
}

#[derive(Debug)]
pub struct Primitive {
    material: Material,
    indices: Option<ElementIndices>,
    positions: VertexAttribute,
    normals: VertexAttribute,
}

impl Primitive {
    pub fn new(
        material: Material,
        indices: Option<ElementIndices>,
        positions: VertexAttribute,
        normals: VertexAttribute,
    ) -> Result<Primitive, ()> {
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
    pub fn draw(&self, program: &ShaderProgram, info: &MaterialShaderInfo) {
        let context: &WebGlRenderingContext = self.positions.buffer.context();
        self.material.set_uniforms(program, info);
        self.positions.bind(info.position.index);
        self.normals.bind(info.normal.index);
        match self.indices {
            Some(ref indices) => {
                indices.bind();
                context.draw_elements_with_i32(
                    WebGlRenderingContext::TRIANGLES,
                    indices.binding.count as i32,
                    indices.binding.index_type as u32,
                    indices.binding.offset as i32,
                );
            }
            None => context.draw_arrays(
                WebGlRenderingContext::TRIANGLES,
                0,
                self.positions.binding.count as i32,
            ),
        }
    }
}

#[derive(Debug)]
pub struct VertexAttribute {
    buffer: Buffer, // TODO: share buffers between primitives.
    binding: VertexAttributeBinding,
}

impl VertexAttribute {
    pub fn new(buffer: Buffer, binding: VertexAttributeBinding) -> VertexAttribute {
        VertexAttribute { buffer, binding }
    }

    /// Binds the primitive's buffers and makes the appropriate WebGL draw call.
    /// The projection and modelview matrix uniforms must already be bound.
    pub fn bind(&self, index: usize) {
        self.buffer.bind_to_attribute(index, &self.binding);
    }
}

#[derive(Debug)]
pub struct ElementIndices {
    buffer: Buffer,
    binding: ElementBinding,
}

impl ElementIndices {
    pub fn new(buffer: Buffer, binding: ElementBinding) -> ElementIndices {
        ElementIndices { buffer, binding }
    }

    pub fn bind(&self) {
        self.buffer.bind();
    }
}
