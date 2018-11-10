use std::convert::TryInto;
use std::rc::Rc;

use gltf;
use gltf::accessor::DataType;
use gltf::buffer::Source;
use gltf::{Accessor, Gltf, Semantic};

use rendering::buffer::{AttributeType, Buffer, ElementBinding, VertexAttributeBinding};
use rendering::context::RenderingContext;
use rendering::material::Material;
use rendering::mesh::{ElementIndices, Mesh, Primitive, VertexAttribute};
use rendering::Rgba;

pub struct GltfLoader<'a, Context: RenderingContext> {
    context: Rc<Context>,
    gltf: &'a Gltf, // TODO: make sure all input parameters come from this Gltf instance?
}

impl<'a, Context> GltfLoader<'a, Context>
where
    Context: RenderingContext,
{
    pub fn new(context: Rc<Context>, gltf: &'a Gltf) -> GltfLoader<'a, Context> {
        GltfLoader { context, gltf }
    }

    pub fn gltf(&self) -> &'a Gltf {
        self.gltf
    }

    pub fn load_attribute(&mut self, accessor: &Accessor) -> Result<VertexAttribute<Context>, ()> {
        if accessor.sparse().is_some() {
            return Err(()); // TODO: support sparse accessors.
        }

        let buffer = self.context.make_attribute_buffer()?; // TODO: share these between attributes.
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

        Ok(VertexAttribute::new(Rc::new(buffer), binding))
    }

    pub fn load_material(material: &gltf::Material) -> Result<Material, ()> {
        let pbr = material.pbr_metallic_roughness();
        Ok(Material {
            base_color: from4(pbr.base_color_factor()),
            metal_factor: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
        })
    }

    pub fn load_mesh(&mut self, mesh: &gltf::Mesh) -> Result<Mesh<Context>, ()> {
        let primitives: Result<Vec<Primitive<Context>>, ()> = mesh
            .primitives()
            .map(|ref p| self.load_primitive(p))
            .collect();
        Ok(Mesh::new(primitives?))
    }

    pub fn load_primitive(
        &mut self,
        primitive: &gltf::Primitive,
    ) -> Result<Primitive<Context>, ()> {
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

    fn load_buffer(&self, gl_buf: &Buffer<RenderingContext = Context>, src_buf: &gltf::Buffer) {
        let blob = self.gltf.blob.as_ref();
        let data = match src_buf.source() {
            Source::Bin => blob.map(|vec| &vec[..]).unwrap_or(&[]),
            Source::Uri(_) => &[], // TODO: implement.
        };
        gl_buf.set_data(&data[0..src_buf.length()]);
    }

    fn load_indices(&mut self, accessor: &Accessor) -> Result<ElementIndices<Context>, ()> {
        let buffer = self.context.make_index_buffer()?;
        let view = accessor.view();
        self.load_buffer(&buffer, &view.buffer());
        let attr_type: AttributeType = accessor.data_type().into();
        Ok(ElementIndices::new(
            Rc::new(buffer),
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