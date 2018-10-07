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
use glue::webgl::mesh::{ElementIndices, Mesh, Primitive, VertexAttribute};
use rendering::material::Material;
use rendering::Rgba;

pub struct GltfLoader<'a> {
    context: Rc<WebGlRenderingContext>,
    gltf: &'a Gltf, // TODO: make sure all input parameters come from this Gltf instance?
}

impl<'a> GltfLoader<'a> {
    pub fn new(context: Rc<WebGlRenderingContext>, gltf: &Gltf) -> GltfLoader {
        GltfLoader { context, gltf }
    }

    pub fn gltf(&self) -> &'a Gltf {
        self.gltf
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

    pub fn load_material(material: &gltf::Material) -> Result<Material, ()> {
        let pbr = material.pbr_metallic_roughness();
        Ok(Material {
            base_color: from4(pbr.base_color_factor()),
            metal_factor: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
        })
    }

    pub fn load_mesh(&mut self, mesh: &gltf::Mesh) -> Result<Mesh, ()> {
        let primitives: Result<Vec<Primitive>, ()> = mesh
            .primitives()
            .map(|ref p| self.load_primitive(p))
            .collect();
        Ok(Mesh::new(primitives?))
    }

    pub fn load_primitive(&mut self, primitive: &gltf::Primitive) -> Result<Primitive, ()> {
        let material = Self::load_material(&primitive.material())?;
        let (_, pos_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Positions)
            .ok_or(())?;
        let (_, normal_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Normals)
            .ok_or(())?;
        let indices = match primitive.indices() {
            Some(ref accessor) => Some(self.load_indices(accessor)?),
            None => None,
        };
        let positions = self.load_attribute(&pos_accessor)?;
        let normals = self.load_attribute(&normal_accessor)?;
        Primitive::new(material, indices, positions, normals)
    }

    fn load_buffer(&self, gl_buf: &Buffer, src_buf: &gltf::Buffer) {
        let blob = self.gltf.blob.as_ref();
        let data = match src_buf.source() {
            Source::Bin => blob.map(|vec| &vec[..]).unwrap_or(&[]),
            Source::Uri(_) => &[], // TODO: implement.
        };
        // TODO: remove the copy.
        gl_buf.set_data(&data[0..src_buf.length()]);
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

fn from4(c: [f32; 4]) -> Rgba {
    Rgba::new(c[0], c[1], c[2], c[3])
}
