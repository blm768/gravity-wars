use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;
use std::slice;

use cgmath::Vector3;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum AttributeType {
    Byte = WebGlRenderingContext::BYTE,
    UnsignedByte = WebGlRenderingContext::UNSIGNED_BYTE,
    Short = WebGlRenderingContext::SHORT,
    UnsignedShort = WebGlRenderingContext::UNSIGNED_SHORT,
    Int = WebGlRenderingContext::INT,
    UnsignedInt = WebGlRenderingContext::UNSIGNED_INT,
    Float = WebGlRenderingContext::FLOAT,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum IndexType {
    UnsignedByte = WebGlRenderingContext::UNSIGNED_BYTE,
    UnsignedShort = WebGlRenderingContext::UNSIGNED_SHORT,
    UnsignedInt = WebGlRenderingContext::UNSIGNED_INT,
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

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum BufferUsage {
    StaticDraw = WebGlRenderingContext::STATIC_DRAW,
    DynamicDraw = WebGlRenderingContext::DYNAMIC_DRAW,
    StreamDraw = WebGlRenderingContext::STREAM_DRAW,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum BufferBinding {
    ArrayBuffer = WebGlRenderingContext::ARRAY_BUFFER,
    ElementArrayBuffer = WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
}

pub trait BufferDataItem: Sized {
    const ATTRIB_TYPE: AttributeType;
    const ATTRIB_COUNT: usize;
}

impl BufferDataItem for u8 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Byte;
    const ATTRIB_COUNT: usize = 1;
}

impl BufferDataItem for f32 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Float;
    const ATTRIB_COUNT: usize = 1;
}

impl<T: BufferDataItem> BufferDataItem for Vector3<T> {
    const ATTRIB_TYPE: AttributeType = <T as BufferDataItem>::ATTRIB_TYPE;
    const ATTRIB_COUNT: usize = 3;
}

pub struct Buffer {
    buffer: WebGlBuffer,
    binding: BufferBinding,
    context: Rc<WebGlRenderingContext>,
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Buffer") // TODO: provide more information.
    }
}

impl Buffer {
    pub fn new(context: Rc<WebGlRenderingContext>, binding: BufferBinding) -> Option<Buffer> {
        let buffer = context.create_buffer()?;
        Some(Buffer {
            buffer,
            binding,
            context,
        })
    }

    pub fn context(&self) -> &Rc<WebGlRenderingContext> {
        &self.context
    }

    pub fn set_data<T: BufferDataItem>(&self, data: &[T]) {
        self.bind();
        // TODO: this may be quite unsafe...
        // (But I think the mutability is a side effect of the bindings; WebGL shouldn't be mutating anything...)
        let bytes = unsafe {
            slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len() * mem::size_of::<T>())
        };
        // TODO: support other hint values.
        self.context.buffer_data_with_u8_array(
            self.binding as u32,
            bytes,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    pub fn bind(&self) {
        self.context
            .bind_buffer(self.binding as u32, Some(&self.buffer));
    }

    // TODO: take a ShaderParamInfo instead of just an index?
    pub fn bind_to_attribute(&self, index: usize, binding: &VertexAttributeBinding) {
        self.bind();
        self.context.vertex_attrib_pointer_with_i32(
            index as u32,
            binding.num_components as i32, // TODO: use the correct value here...
            binding.attr_type as u32,
            binding.normalized,
            binding.stride as i32,
            binding.offset as i32,
        );
        self.context.enable_vertex_attrib_array(index as u32);
    }
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
    pub fn typed<T: BufferDataItem>(count: usize) -> VertexAttributeBinding {
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

#[derive(Clone, Copy, Debug)]
pub struct ElementBinding {
    pub index_type: IndexType,
    pub count: usize,
    pub offset: usize,
}
