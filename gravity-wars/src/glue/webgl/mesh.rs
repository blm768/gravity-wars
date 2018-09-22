use std::convert::TryInto;
use std::rc::Rc;

use gltf;
use gltf::accessor::DataType;
use gltf::buffer::Source;
use gltf::{Accessor, Gltf, Semantic};
use web_sys::WebGlRenderingContext;

use glue::webgl::buffer::{
    AttributeType, Buffer, BufferBinding, ElementBinding, VertexAttributeBinding,
};
use rendering::shader::MaterialShaderInfo;

pub struct GltfLoader<'a> {
    context: Rc<WebGlRenderingContext>,
    gltf: &'a Gltf, // TODO: make sure all input parameters come from this Gltf instance?
}

impl<'a> GltfLoader<'a> {
    pub fn new(context: Rc<WebGlRenderingContext>, gltf: &Gltf) -> GltfLoader {
        GltfLoader { context, gltf }
    }

    pub fn first_mesh(&self) -> Option<gltf::Mesh<'a>> {
        self.gltf.meshes().next()
    }

    pub fn load_attribute(&mut self, accessor: &Accessor) -> Result<VertexAttribute, ()> {
        if accessor.sparse().is_some() {
            return Err(()); // TODO: support sparse accessors.
        }

        let buffer = Buffer::new(self.context.clone(), BufferBinding::ArrayBuffer).ok_or(())?;
        let view = accessor.view();
        let src_buf = view.buffer();

        self.load_buffer(&buffer, &src_buf);

        let binding = VertexAttributeBinding {
            attr_type: AttributeType::Float, // TODO: get from accessor.
            num_components: accessor.dimensions().multiplicity(),
            normalized: accessor.normalized(),
            stride: view.stride().unwrap_or(0),
            offset: view.offset() + accessor.offset(),
            count: accessor.count(),
        };

        Ok(VertexAttribute::new(buffer, binding))
    }

    pub fn load_mesh(&mut self, mesh: &gltf::Mesh) -> Result<Mesh, ()> {
        let primitives: Result<Vec<Primitive>, ()> = mesh
            .primitives()
            .map(|ref p| self.load_primitive(p))
            .collect();
        Ok(Mesh::new(primitives?))
    }

    pub fn load_primitive(&mut self, primitive: &gltf::Primitive) -> Result<Primitive, ()> {
        let (_, pos_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Positions)
            .ok_or(())?;
        let indices = match primitive.indices() {
            Some(ref accessor) => Some(self.load_indices(accessor)?),
            None => None,
        };
        let position = self.load_attribute(&pos_accessor)?;
        Primitive::new(indices, position)
    }

    fn load_buffer(&self, gl_buf: &Buffer, src_buf: &gltf::Buffer) {
        let blob = self.gltf.blob.as_ref();
        let data = match src_buf.source() {
            Source::Bin => blob.map(|vec| &vec[..]).unwrap_or(&[]),
            Source::Uri(_) => &[], // TODO: implement.
        };
        // TODO: remove the copy.
        gl_buf.set_data(&mut Vec::from(&data[0..src_buf.length()]));
    }

    fn load_indices(&mut self, accessor: &Accessor) -> Result<ElementIndices, ()> {
        let buffer =
            Buffer::new(self.context.clone(), BufferBinding::ElementArrayBuffer).ok_or(())?;
        let view = accessor.view();
        self.load_buffer(&buffer, &view.buffer());
        let attr_type: AttributeType = accessor.data_type().into();
        Ok(ElementIndices::new(
            buffer,
            ElementBinding {
                count: accessor.count(),
                index_type: attr_type.try_into()?,
                offset: view.offset() + accessor.offset(),
            },
        ))
    }
}

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

    pub fn draw(&self, info: &MaterialShaderInfo) {
        for p in self.primitives.iter() {
            p.draw(info);
        }
    }
}

#[derive(Debug)]
pub struct Primitive {
    indices: Option<ElementIndices>,
    position: VertexAttribute,
}

impl Primitive {
    pub fn new(
        indices: Option<ElementIndices>,
        position: VertexAttribute,
    ) -> Result<Primitive, ()> {
        Ok(Primitive { indices, position })
    }

    /// Binds each primitive's buffers and makes the appropriate WebGL draw calls.
    /// The projection and modelview matrix uniforms must already be bound.
    // TODO: break out a separate bind() method?
    pub fn draw(&self, info: &MaterialShaderInfo) {
        let context: &WebGlRenderingContext = self.position.buffer.context();
        self.position.bind(info.position.index);
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
                self.position.binding.count as i32,
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

impl From<DataType> for AttributeType {
    fn from(data_type: gltf::accessor::DataType) -> Self {
        match data_type {
            DataType::I8 => AttributeType::Byte,
            DataType::U8 => AttributeType::UnsignedByte,
            DataType::I16 => AttributeType::Short,
            DataType::U16 => AttributeType::UnsignedShort,
            DataType::U32 => AttributeType::UnsignedInt,
            DataType::F32 => AttributeType::Float,
        }
    }
}
