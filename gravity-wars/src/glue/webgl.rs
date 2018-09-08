use wasm_bindgen::prelude::*;

use std::collections::HashMap;
use std::error::Error;
use std::mem;
use std::rc::Rc;
use std::slice;

use cgmath::{Matrix4, Vector3};

use glue;
use rendering::renderer::GameRenderer;
use rendering::shader;
use rendering::shader::ShaderParamInfo;
use state::GameState;

pub const DEPTH_BUFFER_BIT: i32 = 0x0100;
pub const STENCIL_BUFFER_BIT: i32 = 0x0400;
pub const COLOR_BUFFER_BIT: i32 = 0x4000;

pub const ARRAY_BUFFER: i32 = 0x8892;
pub const ELEMENT_ARRAY_BUFFER: i32 = 0x8893;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum BufferBinding {
    ArrayBuffer = ARRAY_BUFFER,
    ElementArrayBuffer = ELEMENT_ARRAY_BUFFER,
}

pub const STATIC_DRAW: i32 = 0x88E4;
pub const DYNAMIC_DRAW: i32 = 0x88E8;
pub const STREAM_DRAW: i32 = 0x88E0;

pub const FLOAT: i32 = 0x1406;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum AttributeType {
    Float = FLOAT,
}

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum BufferUsage {
    StaticDraw = STATIC_DRAW,
    DynamicDraw = DYNAMIC_DRAW,
    StreamDraw = STREAM_DRAW,
}

pub const VERTEX_SHADER: i32 = 0x8B31;
pub const FRAGMENT_SHADER: i32 = 0x8B30;
pub const COMPILE_STATUS: i32 = 0x8B81;
pub const LINK_STATUS: i32 = 0x8B82;
pub const ACTIVE_ATTRIBUTES: i32 = 0x8B89;
pub const ACTIVE_UNIFORMS: i32 = 0x8B86;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex = VERTEX_SHADER,
    Fragment = FRAGMENT_SHADER,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=glue, js_name=getWebGLContext)]
    pub fn get_webgl_context() -> WebGLRenderingContext;

    pub type WebGLRenderingContext;
    #[wasm_bindgen(method, js_name=clearColor)]
    pub fn clear_color(context: &WebGLRenderingContext, r: f32, g: f32, b: f32, a: f32);
    #[wasm_bindgen(method)]
    pub fn clear(context: &WebGLRenderingContext, mask: i32);
    #[wasm_bindgen(method, catch)]
    pub fn vertex_attrib_pointer(
        context: &WebGLRenderingContext,
        index: u32,
        size: i32,
        attr_type: i32,
        normalized: bool,
        stride: u32,
        offset: u32, // TODO: move to u64 when it no longer breaks wasm-bindgen.
    ) -> Result<(), JsValue>;

    type WebGLBuffer;
    #[wasm_bindgen(method, js_name=createBuffer)]
    fn create_buffer(context: &WebGLRenderingContext) -> WebGLBuffer;
    #[wasm_bindgen(method, js_name=bindBuffer)]
    fn bind_buffer(
        context: &WebGLRenderingContext,
        binding: i32,
        buffer: &WebGLBuffer,
    ) -> WebGLBuffer;
    #[wasm_bindgen(method, js_name=bufferData)]
    fn buffer_data(
        context: &WebGLRenderingContext,
        target: i32,
        data: &[u8],
        usage: i32,
    ) -> WebGLBuffer;

    pub type WebGLShader;
    #[wasm_bindgen(method, js_name=createShader)]
    pub fn create_shader(context: &WebGLRenderingContext, shader_type: i32) -> WebGLShader;
    #[wasm_bindgen(method, js_name=shaderSource)]
    pub fn shader_source(context: &WebGLRenderingContext, shader: &WebGLShader, source: &str);
    #[wasm_bindgen(method, js_name=compileShader)]
    pub fn compile_shader(context: &WebGLRenderingContext, shader: &WebGLShader);
    #[wasm_bindgen(method, js_name=getShaderParameter)]
    pub fn get_shader_parameter_boolean(
        context: &WebGLRenderingContext,
        shader: &WebGLShader,
        param: i32,
    ) -> bool;
    #[wasm_bindgen(method, js_name=getShaderInfoLog)]
    pub fn get_shader_info_log(context: &WebGLRenderingContext, shader: &WebGLShader) -> String;

    pub type WebGLProgram;
    #[wasm_bindgen(method, js_name=createProgram)]
    pub fn create_program(context: &WebGLRenderingContext) -> WebGLProgram;
    #[wasm_bindgen(method, js_name=attachShader)]
    pub fn attach_shader(
        context: &WebGLRenderingContext,
        program: &WebGLProgram,
        shader: &WebGLShader,
    );
    #[wasm_bindgen(method, js_name=linkProgram)]
    pub fn link_program(context: &WebGLRenderingContext, program: &WebGLProgram);
    #[wasm_bindgen(method, js_name=getProgramParameter)]
    pub fn get_program_parameter_boolean(
        context: &WebGLRenderingContext,
        program: &WebGLProgram,
        param: i32,
    ) -> bool;
    #[wasm_bindgen(method, js_name=getProgramParameter)]
    pub fn get_program_parameter_i32(
        context: &WebGLRenderingContext,
        program: &WebGLProgram,
        param: i32,
    ) -> i32;
    #[wasm_bindgen(method, js_name=getProgramInfoLog)]
    pub fn get_program_info_log(context: &WebGLRenderingContext, program: &WebGLProgram) -> String;
    #[wasm_bindgen(method, js_name=useProgram)]
    pub fn use_program(context: &WebGLRenderingContext, program: &WebGLProgram);

    type WebGLActiveInfo;
    #[wasm_bindgen(method, js_name=getActiveAttrib)]
    fn get_active_attribute(
        context: &WebGLRenderingContext,
        program: &WebGLProgram,
        index: u32,
    ) -> WebGLActiveInfo;
    #[wasm_bindgen(method, js_name=getActiveUniform)]
    fn get_active_uniform(
        context: &WebGLRenderingContext,
        program: &WebGLProgram,
        index: u32,
    ) -> WebGLActiveInfo;
    #[wasm_bindgen(method, getter)]
    fn name(info: &WebGLActiveInfo) -> String;
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

pub struct Buffer {
    buffer: WebGLBuffer,
    binding: BufferBinding,
    context: Rc<WebGLRenderingContext>,
}

impl Buffer {
    pub fn new(context: Rc<WebGLRenderingContext>, binding: BufferBinding) -> Buffer {
        let buffer = context.create_buffer();
        Buffer {
            buffer,
            binding,
            context,
        }
    }

    pub fn set_data<T: BufferDataItem>(&self, data: &[T]) {
        self.bind();
        let bytes = unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<T>())
        };
        // TODO: support other hint values.
        self.context
            .buffer_data(self.binding as i32, bytes, STATIC_DRAW);
    }

    // TODO: take a ShaderParamInfo instead of just an index?
    pub fn bind_to_attribute(&self, index: usize, binding: &VertexAttributeBinding) {
        self.bind();
        self.context
            .vertex_attrib_pointer(
                index as u32,
                binding.num_components as i32,
                binding.attr_type as i32,
                binding.normalized,
                binding.stride as u32,
                binding.offset as u32,
            )
            .or_else(|e| {
                glue::log(e.to_string());
                Ok(())
            });
    }

    fn bind(&self) {
        self.context.bind_buffer(self.binding as i32, &self.buffer);
    }
}

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

impl WebGLShader {
    pub fn compile(
        context: &WebGLRenderingContext,
        shader_type: ShaderType,
        source: &str,
    ) -> Result<WebGLShader, String> {
        let shader = context.create_shader(shader_type as i32);
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context.get_shader_parameter_boolean(&shader, COMPILE_STATUS) {
            Ok(shader)
        } else {
            Err(context.get_shader_info_log(&shader))
        }
    }
}

pub struct ShaderProgram {
    program: WebGLProgram,
    context: Rc<WebGLRenderingContext>,
}

impl ShaderProgram {
    pub fn link<'a, T: Iterator<Item = &'a WebGLShader>>(
        context: Rc<WebGLRenderingContext>,
        shaders: T,
    ) -> Result<ShaderProgram, String> {
        let program = context.create_program();
        for ref shader in shaders {
            context.attach_shader(&program, shader)
        }
        context.link_program(&program);

        if context.get_program_parameter_boolean(&program, LINK_STATUS) {
            Ok(ShaderProgram { program, context })
        } else {
            Err(context.get_program_info_log(&program))
        }
    }
}

impl shader::ShaderProgram for ShaderProgram {
    fn attributes(&self) -> HashMap<Box<str>, ShaderParamInfo> {
        let num_attributes =
            self.context
                .get_program_parameter_i32(&self.program, ACTIVE_ATTRIBUTES) as u32;
        let mut attributes = HashMap::<Box<str>, ShaderParamInfo>::new();
        for i in 0..num_attributes {
            let info = self.context.get_active_attribute(&self.program, i);
            attributes.insert(
                info.name().into_boxed_str(),
                ShaderParamInfo { index: i as usize },
            );
        }
        attributes
    }

    fn uniforms(&self) -> HashMap<Box<str>, ShaderParamInfo> {
        let num_uniforms =
            self.context
                .get_program_parameter_i32(&self.program, ACTIVE_UNIFORMS) as u32;
        let mut uniforms = HashMap::<Box<str>, ShaderParamInfo>::new();
        for i in 0..num_uniforms {
            let info = self.context.get_active_uniform(&self.program, i);
            uniforms.insert(
                info.name().into_boxed_str(),
                ShaderParamInfo { index: i as usize },
            );
        }
        uniforms
    }

    fn activate(&self) {
        self.context.use_program(&self.program);
    }
}

impl WebGLRenderingContext {
    pub fn new() -> WebGLRenderingContext {
        get_webgl_context()
    }
}

pub struct WebGlRenderer {
    context: Rc<WebGLRenderingContext>,
    aspect_ratio: f32,
}

impl WebGlRenderer {
    pub fn new() -> WebGlRenderer {
        let context = Rc::new(WebGLRenderingContext::new());
        WebGlRenderer {
            context,
            aspect_ratio: 1.0,
        }
    }

    pub fn context(&self) -> &Rc<WebGLRenderingContext> {
        &self.context
    }
}

impl GameRenderer for WebGlRenderer {
    fn render(&self, state: &GameState) -> Result<(), Box<Error>> {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(COLOR_BUFFER_BIT);

        let projection: Matrix4<f32> = state.camera.projection(self.aspect_ratio).into();

        // TODO: implement.
        for entity in &state.map.entities {
            let modelview_transform = Matrix4::<f32>::from_translation(entity.position.extend(0.0));
        }

        Ok(())
    }
}
