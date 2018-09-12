use std::mem;
use std::rc::Rc;
use std::slice;

use cgmath::Vector3;
use wasm_bindgen::prelude::*;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum AttributeType {
    Float = WebGlRenderingContext::FLOAT,
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

impl BufferDataItem for f32 {
    const ATTRIB_TYPE: AttributeType = AttributeType::Float;
    const ATTRIB_COUNT: usize = 1;
}

impl<T: BufferDataItem> BufferDataItem for Vector3<T> {
    const ATTRIB_TYPE: AttributeType = <T as BufferDataItem>::ATTRIB_TYPE;
    const ATTRIB_COUNT: usize = 3;
}

#[wasm_bindgen(module = "./glue")]
extern "C" {
    // Wraps vertexAttribPointer to take a u32 offset instead of i64 (which isn't what JS seems to actually be expecting…)
    fn my_vertex_attrib_pointer(
        context: &WebGlRenderingContext,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: u32,
    );
}

pub struct Buffer {
    buffer: WebGlBuffer,
    binding: BufferBinding,
    context: Rc<WebGlRenderingContext>,
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

    pub fn set_data<T: BufferDataItem>(&self, data: &mut [T]) {
        self.bind();
        // TODO: this is probably quite unsafe...
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

    // TODO: take a ShaderParamInfo instead of just an index?
    pub fn bind_to_attribute(&self, index: usize, binding: &VertexAttributeBinding) {
        self.bind();
        my_vertex_attrib_pointer(
            &self.context,
            index as u32,
            binding.num_components as i32,
            binding.attr_type as u32,
            binding.normalized,
            binding.stride as i32,
            binding.offset as u32,
        );
        self.context.enable_vertex_attrib_array(index as u32);
    }

    fn bind(&self) {
        self.context
            .bind_buffer(self.binding as u32, Some(&self.buffer));
    }
}

#[derive(Clone, Debug)]
pub struct VertexAttributeBinding {
    pub attr_type: AttributeType,
    pub num_components: usize,
    pub normalized: bool,
    pub stride: usize,
    pub offset: usize,
}

impl VertexAttributeBinding {
    pub fn typed<T: BufferDataItem>() -> VertexAttributeBinding {
        VertexAttributeBinding {
            attr_type: T::ATTRIB_TYPE,
            num_components: T::ATTRIB_COUNT,
            normalized: false,
            stride: 0,
            offset: 0,
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
