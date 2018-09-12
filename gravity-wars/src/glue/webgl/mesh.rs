use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use gltf;
use gltf::{Gltf, Semantic};
use web_sys::WebGlRenderingContext;

use glue::webgl::buffer::{Buffer, BufferBinding};

pub fn find_mesh(gltf: &Gltf) -> Option<gltf::Mesh> {
    gltf.meshes().next()
}

#[derive(Debug)]
pub struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn from_gltf(context: Rc<WebGlRenderingContext>, mesh: gltf::Mesh) -> Result<Mesh, ()> {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            primitives.push(Primitive::from_gltf(context.clone(), primitive)?);
        }
        Ok(Mesh { primitives })
    }
}

#[derive(Debug)]
pub struct Primitive {
    position: VertexAttribute,
}

impl Primitive {
    pub fn from_gltf(
        context: Rc<WebGlRenderingContext>,
        primitive: gltf::Primitive,
    ) -> Result<Primitive, ()> {
        let (_, pos_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Positions)
            .ok_or(())?;
        let position = VertexAttribute::from_gltf(context, pos_accessor)?;

        Ok(Primitive { position })
    }
}

pub struct VertexAttribute {
    buffer: Buffer, // TODO: share buffers between primitives.
}

impl VertexAttribute {
    pub fn from_gltf(
        context: Rc<WebGlRenderingContext>,
        accessor: gltf::Accessor,
    ) -> Result<VertexAttribute, ()> {
        let buffer = Buffer::new(context, BufferBinding::ArrayBuffer).ok_or(())?;
        Ok(VertexAttribute { buffer })
    }
}

// TODO: implement better.
impl Debug for VertexAttribute {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VertexAttribute")
    }
}
