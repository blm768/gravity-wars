use std::convert::TryFrom;
use std::fmt::Debug;
use std::mem;
use std::slice;

use nalgebra::Vector3;

use crate::rendering::context;

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

pub trait BufferData<E: VertexAttributeData> {
    fn as_bytes(&self) -> &[u8];
    fn num_elements(&self) -> usize;
}

impl<T, E> BufferData<E> for T
where
    E: VertexAttributeData,
    T: AsRef<[E]>,
{
    fn as_bytes(&self) -> &[u8] {
        let array: &[E] = self.as_ref();
        unsafe {
            slice::from_raw_parts(
                array.as_ptr() as *const u8,
                array.len() * mem::size_of::<E>(),
            )
        }
    }

    fn num_elements(&self) -> usize {
        self.as_ref().len()
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

/// Marker trait for types that can be used as attribute data for rendering buffers
///
/// This trait is marked as unsafe because of the blanket impl of BufferData for
/// values that can be converted to `&[T] where T: VertexAttributeData`; that
/// impl assumes that an array of T can be safely reinterpreted as an array of
/// `u8`.
pub unsafe trait VertexAttributeData: Sized + 'static {
    const ATTRIB_TYPE: AttributeType;
    const ATTRIB_COUNT: usize;
}

/// Marker trait for types that can be used as index data for rendering buffers
pub unsafe trait VertexIndexData: VertexAttributeData + Sized + 'static {
    const INDEX_TYPE: IndexType;
}

/* TODO: use a blanket impl (and remove the VertexAttributeData bound on VertexIndexData) instead of the manual impls once const fn supports matching.
unsafe impl<T: VertexIndexData> VertexAttributeData for T {
    const ATTRIB_TYPE: AttributeType = index_type_to_attribute_type(T::INDEX_TYPE);
    const ATTRIB_COUNT: usize = 1;
}
*/

unsafe impl VertexAttributeData for u8 {
    const ATTRIB_TYPE: AttributeType = AttributeType::UnsignedByte;
    const ATTRIB_COUNT: usize = 1;
}

unsafe impl VertexIndexData for u8 {
    const INDEX_TYPE: IndexType = IndexType::UnsignedByte;
}

unsafe impl VertexAttributeData for u16 {
    const ATTRIB_TYPE: AttributeType = AttributeType::UnsignedShort;
    const ATTRIB_COUNT: usize = 1;
}

unsafe impl VertexIndexData for u16 {
    const INDEX_TYPE: IndexType = IndexType::UnsignedShort;
}

unsafe impl VertexAttributeData for u32 {
    const ATTRIB_TYPE: AttributeType = AttributeType::UnsignedInt;
    const ATTRIB_COUNT: usize = 1;
}

unsafe impl VertexIndexData for u32 {
    const INDEX_TYPE: IndexType = IndexType::UnsignedInt;
}

unsafe impl VertexAttributeData for f32 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Float;
    const ATTRIB_COUNT: usize = 1;
}

// TODO: split out vector/scalar traits so we can't try to make vectors of vectors?
unsafe impl<T> VertexAttributeData for Vector3<T>
where
    T: VertexAttributeData + nalgebra::Scalar,
{
    const ATTRIB_TYPE: AttributeType = <T as VertexAttributeData>::ATTRIB_TYPE;
    const ATTRIB_COUNT: usize = 3;
}
