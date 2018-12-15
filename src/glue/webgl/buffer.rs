use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::slice;

use web_sys::{WebGlBuffer, WebGlRenderingContext};

use crate::glue::webgl::WebGlContext;
use crate::rendering;
use crate::rendering::buffer::{AttributeType, VertexAttributeBinding};

fn to_gl_attr_type(attr_type: AttributeType) -> u32 {
    match attr_type {
        AttributeType::Byte => WebGlRenderingContext::BYTE,
        AttributeType::UnsignedByte => WebGlRenderingContext::UNSIGNED_BYTE,
        AttributeType::Short => WebGlRenderingContext::SHORT,
        AttributeType::UnsignedShort => WebGlRenderingContext::UNSIGNED_SHORT,
        AttributeType::Int => WebGlRenderingContext::INT,
        AttributeType::UnsignedInt => WebGlRenderingContext::UNSIGNED_INT,
        AttributeType::Float => WebGlRenderingContext::FLOAT,
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
enum BufferBinding {
    ArrayBuffer = WebGlRenderingContext::ARRAY_BUFFER,
    ElementArrayBuffer = WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
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
    fn new(context: Rc<WebGlRenderingContext>, binding: BufferBinding) -> Option<Buffer> {
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

    fn bind(&self) {
        self.context
            .bind_buffer(self.binding as u32, Some(&self.buffer));
    }

    fn set_data(&self, data: &[u8]) {
        self.bind();
        // TODO: this may be quite unsafe...
        // (But I think the mutability is a side effect of the bindings; WebGL shouldn't be mutating anything...)
        let bytes = unsafe { slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len()) };
        // TODO: support other hint values.
        self.context.buffer_data_with_u8_array(
            self.binding as u32,
            bytes,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
}

#[derive(Debug)]
pub struct AttributeBuffer {
    buffer: Buffer,
}

impl AttributeBuffer {
    pub fn new(context: Rc<WebGlRenderingContext>) -> Option<Self> {
        Some(AttributeBuffer {
            buffer: Buffer::new(context, BufferBinding::ArrayBuffer)?,
        })
    }
}

impl rendering::buffer::Buffer for AttributeBuffer {
    type RenderingContext = WebGlContext;

    fn set_data(&self, data: &[u8]) {
        self.buffer.set_data(data);
    }
}

impl rendering::buffer::AttributeBuffer for AttributeBuffer {
    fn bind_to_attribute(&self, index: usize, binding: &VertexAttributeBinding) {
        self.buffer.bind();
        self.buffer.context.vertex_attrib_pointer_with_i32(
            index as u32,
            binding.num_components as i32,
            to_gl_attr_type(binding.attr_type),
            binding.normalized,
            binding.stride as i32,
            binding.offset as i32,
        );
        self.buffer.context.enable_vertex_attrib_array(index as u32);
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    buffer: Buffer,
}

impl IndexBuffer {
    pub fn new(context: Rc<WebGlRenderingContext>) -> Option<Self> {
        Some(IndexBuffer {
            buffer: Buffer::new(context, BufferBinding::ElementArrayBuffer)?,
        })
    }

    pub fn bind(&self) {
        self.buffer.bind();
    }
}

impl rendering::buffer::Buffer for IndexBuffer {
    type RenderingContext = WebGlContext;

    fn set_data(&self, data: &[u8]) {
        self.buffer.set_data(data);
    }
}

impl rendering::buffer::IndexBuffer for IndexBuffer {}
