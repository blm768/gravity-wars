use std::convert::TryFrom;
use std::fmt::Debug;
use std::mem;
use std::slice;

use cgmath::Vector3;

use rendering::context;

#[derive(Clone, Copy, Debug)]
pub enum AttributeType {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Float,
}

#[derive(Clone, Copy, Debug)]
pub enum IndexType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

impl TryFrom<AttributeType> for IndexType {
    type Error = ();
    fn try_from(attr_type: AttributeType) -> Result<Self, ()> {
        match attr_type {
            AttributeType::UnsignedByte => Ok(IndexType::UnsignedByte),
            AttributeType::UnsignedShort => Ok(IndexType::UnsignedShort),
            AttributeType::UnsignedInt => Ok(IndexType::UnsignedInt),
            _ => Err(()),
        }
    }
}

pub trait Buffer: Debug {
    type RenderingContext: context::RenderingContext + ?Sized;

    fn set_data(&self, data: &[u8]);
}

pub trait AttributeBuffer: Buffer {
    // TODO: take a ShaderParamInfo instead of just an index?
    fn bind_to_attribute(&self, index: usize, binding: &VertexAttributeBinding);
}

pub trait IndexBuffer: Buffer {}

pub trait BufferData {
    fn as_bytes(&self) -> &[u8];
}

impl<T: VertexAttributeData> BufferData for &[T] {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * mem::size_of::<T>())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ElementBinding {
    pub index_type: IndexType,
    pub count: usize,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexAttributeBinding {
    pub attr_type: AttributeType,
    pub num_components: usize,
    pub normalized: bool,
    pub stride: usize,
    pub offset: usize,
    pub count: usize,
}

impl VertexAttributeBinding {
    pub fn typed<T: VertexAttributeData>(count: usize) -> VertexAttributeBinding {
        VertexAttributeBinding {
            attr_type: T::ATTRIB_TYPE,
            num_components: T::ATTRIB_COUNT,
            normalized: false,
            stride: 0,
            offset: 0,
            count,
        }
    }

    pub fn set_normalized(&mut self, normalized: bool) -> &mut VertexAttributeBinding {
        self.normalized = normalized;
        self
    }

    pub fn set_stride(&mut self, stride: usize) -> &mut VertexAttributeBinding {
        self.stride = stride;
        self
    }

    pub fn set_offset(&mut self, offset: usize) -> &mut VertexAttributeBinding {
        self.offset = offset;
        self
    }
}

pub trait VertexAttributeData: Sized {
    const ATTRIB_TYPE: AttributeType;
    const ATTRIB_COUNT: usize;
}

impl VertexAttributeData for u8 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Byte;
    const ATTRIB_COUNT: usize = 1;
}

impl VertexAttributeData for f32 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Float;
    const ATTRIB_COUNT: usize = 1;
}

// TODO: split out vector/scalar traits so we can't try to make vectors of vectors?
impl<T: VertexAttributeData> VertexAttributeData for Vector3<T> {
    const ATTRIB_TYPE: AttributeType = <T as VertexAttributeData>::ATTRIB_TYPE;
    const ATTRIB_COUNT: usize = 3;
}
