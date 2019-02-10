use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use nalgebra::{Matrix4, Vector3, Vector4};
use web_sys::{Element, HtmlCanvasElement};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};

use crate::rendering::buffer::IndexType;
use crate::rendering::context::RenderingContext;
use crate::rendering::mesh::ElementIndices;
use crate::rendering::shader;
use crate::rendering::shader::{BoundShader, ShaderBindError, ShaderParamInfo};

pub mod buffer;
pub mod game_renderer;
pub mod texture;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex = WebGlRenderingContext::VERTEX_SHADER,
    Fragment = WebGlRenderingContext::FRAGMENT_SHADER,
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: ShaderType,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type as u32)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error".into()))
    }
}

#[derive(Debug)]
struct ShaderUniformInformation {
    name: Box<str>,
    location: WebGlUniformLocation,
}

#[derive(Debug)]
pub struct ShaderProgram {
    program: WebGlProgram,
    context: Rc<WebGlRenderingContext>,
    attributes: HashMap<Box<str>, ShaderParamInfo>,
    uniforms: Vec<ShaderUniformInformation>,
}

impl ShaderProgram {
    pub fn link<'a, T: Iterator<Item = &'a WebGlShader>>(
        context: Rc<WebGlRenderingContext>,
        shaders: T,
    ) -> Result<ShaderProgram, String> {
        let program = context
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        for shader in shaders {
            context.attach_shader(&program, shader)
        }
        context.link_program(&program);

        if context
            .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let attributes = ShaderProgram::get_attribute_info(&context, &program);
            let uniforms = ShaderProgram::get_uniform_info(&context, &program);
            Ok(ShaderProgram {
                program,
                context,
                attributes,
                uniforms,
            })
        } else {
            Err(context
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown error creating program object".into()))
        }
    }

    pub fn bind(&self) {
        self.context.use_program(Some(&self.program))
    }

    fn get_attribute_info(
        context: &WebGlRenderingContext,
        program: &WebGlProgram,
    ) -> HashMap<Box<str>, ShaderParamInfo> {
        let num_attributes = context
            .get_program_parameter(program, WebGlRenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .unwrap() as u32;
        let mut attributes = HashMap::<Box<str>, ShaderParamInfo>::new();
        for i in 0..num_attributes {
            if let Some(info) = context.get_active_attrib(program, i) {
                // TODO: log errors?
                attributes.insert(
                    info.name().into_boxed_str(),
                    ShaderParamInfo { index: i as usize },
                );
            }
        }
        attributes
    }

    fn get_uniform_info(
        context: &WebGlRenderingContext,
        program: &WebGlProgram,
    ) -> Vec<ShaderUniformInformation> {
        let num_uniforms = context
            .get_program_parameter(program, WebGlRenderingContext::ACTIVE_UNIFORMS)
            .as_f64()
            .unwrap() as u32; // TODO: create (and use) a safe conversion helper...
        let mut uniforms = Vec::<ShaderUniformInformation>::new();
        uniforms.reserve_exact(num_uniforms as usize);

        for i in 0..num_uniforms {
            // TODO: log errors?
            if let Some(info) = context.get_active_uniform(program, i) {
                let name: Box<str> = info.name().into();
                if let Some(location) = context.get_uniform_location(&program, &name) {
                    uniforms.push(ShaderUniformInformation { name, location });
                }
            }
        }
        uniforms
    }
}

impl shader::ShaderProgram for ShaderProgram {
    type RenderingContext = WebGlContext;

    fn attribute_names(&self) -> Vec<String> {
        self.attributes
            .keys()
            .map(|e| String::from(e.as_ref()))
            .collect::<Vec<_>>()
    }

    fn uniform_names(&self) -> Vec<String> {
        self.uniforms
            .iter()
            .map(|u| String::from(u.name.as_ref()))
            .collect::<Vec<_>>()
    }

    fn attribute(&self, name: &str) -> Option<ShaderParamInfo> {
        self.attributes.get(name).cloned()
    }

    fn uniform(&self, name: &str) -> Option<ShaderParamInfo> {
        self.uniforms
            .iter()
            .enumerate()
            .find(|(_, inf)| inf.name.as_ref() == name)
            .map(|(i, _)| ShaderParamInfo { index: i })
    }
}

#[derive(Debug)]
pub struct WebGlContext {
    canvas_element: Element,
    canvas: HtmlCanvasElement,
    gl_context: Rc<WebGlRenderingContext>,
    shader_bound: Cell<bool>,
}

impl WebGlContext {
    pub fn new(
        canvas_element: Element,
        canvas: HtmlCanvasElement,
        gl_context: WebGlRenderingContext,
    ) -> WebGlContext {
        WebGlContext {
            canvas_element,
            canvas,
            gl_context: Rc::new(gl_context),
            shader_bound: Cell::new(false),
        }
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn gl_context(&self) -> &Rc<WebGlRenderingContext> {
        &self.gl_context
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.gl_context.drawing_buffer_width() as f32
            / self.gl_context.drawing_buffer_height() as f32
    }

    pub fn width(&self) -> i32 {
        self.gl_context.drawing_buffer_width()
    }

    pub fn height(&self) -> i32 {
        self.gl_context.drawing_buffer_height()
    }

    pub fn set_viewport(&self) {
        use std::cmp::max;

        let width = max(self.canvas_element.client_width(), 0) as u32;
        let height = max(self.canvas_element.client_height(), 0) as u32;

        if self.canvas.width() != width {
            self.canvas.set_width(width);
        }
        if self.canvas.height() != height {
            self.canvas.set_height(height);
        }

        self.gl_context.viewport(
            0,
            0,
            self.gl_context.drawing_buffer_width(),
            self.gl_context.drawing_buffer_height(),
        );
    }
}

fn to_gl_element_type(attr_type: IndexType) -> u32 {
    match attr_type {
        IndexType::UnsignedByte => WebGlRenderingContext::UNSIGNED_BYTE,
        IndexType::UnsignedShort => WebGlRenderingContext::UNSIGNED_SHORT,
        IndexType::UnsignedInt => WebGlRenderingContext::UNSIGNED_INT,
    }
}

impl RenderingContext for WebGlContext {
    type AttributeBuffer = buffer::AttributeBuffer;
    type IndexBuffer = buffer::IndexBuffer;
    type ShaderProgram = ShaderProgram;
    type BoundShader = WebGlBoundShader;
    type Texture = texture::Texture;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()> {
        Self::AttributeBuffer::new(self.gl_context.clone()).ok_or(())
    }

    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()> {
        Self::IndexBuffer::new(self.gl_context.clone()).ok_or(())
    }

    fn make_texture(&self) -> Result<Self::Texture, ()> {
        Self::Texture::new(self.gl_context.clone()).ok_or(())
    }

    fn bind_shader(
        &self,
        shader: Rc<Self::ShaderProgram>,
    ) -> Result<Self::BoundShader, ShaderBindError> {
        if self.shader_bound.get() {
            return Err(ShaderBindError::CannotBindMoreShaders);
        }
        if !Rc::ptr_eq(&shader.context, &self.gl_context) {
            return Err(ShaderBindError::InvalidContextForShader);
        }

        shader.bind();
        Ok(WebGlBoundShader {
            context: Rc::clone(&self.gl_context),
            shader,
        })
    }
}

pub struct WebGlBoundShader {
    context: Rc<WebGlRenderingContext>,
    shader: Rc<ShaderProgram>,
}

impl BoundShader<WebGlContext> for WebGlBoundShader {
    fn program(&self) -> &ShaderProgram {
        &self.shader
    }

    fn draw_triangles(&self, count: usize) {
        self.context
            .draw_arrays(WebGlRenderingContext::TRIANGLES, 0, count as i32);
    }

    fn draw_indexed_triangles(&self, indices: &ElementIndices<WebGlContext>) {
        indices.buffer.bind();
        self.context.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            indices.binding.count as i32,
            to_gl_element_type(indices.binding.index_type),
            indices.binding.offset as i32,
        );
    }

    fn draw_polyline(&self, num_vertices: usize) {
        self.context
            .draw_arrays(WebGlRenderingContext::LINE_STRIP, 0, num_vertices as i32);
    }

    fn set_uniform_f32(&self, index: usize, value: f32) {
        self.context
            .uniform1f(Some(&self.shader.uniforms[index].location), value);
    }

    fn set_uniform_mat4(&self, index: usize, mut value: Matrix4<f32>) {
        let raw: &mut [f32] = value.as_mut_slice();
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.shader.uniforms[index].location),
            false,
            raw,
        );
    }

    fn set_uniform_vec3(&self, index: usize, mut value: Vector3<f32>) {
        let raw: &mut [f32; 3] = value.as_mut();
        self.context
            .uniform3fv_with_f32_array(Some(&self.shader.uniforms[index].location), raw);
    }

    fn set_uniform_vec4(&self, index: usize, mut value: Vector4<f32>) {
        let raw: &mut [f32; 4] = value.as_mut();
        self.context
            .uniform4fv_with_f32_array(Some(&self.shader.uniforms[index].location), raw);
    }
}
