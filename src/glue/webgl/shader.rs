use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::rc::Rc;

use nalgebra::{Matrix4, Vector3, Vector4};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};

use crate::glue::webgl::WebGlContext;
use crate::rendering::buffer::IndexType;
use crate::rendering::mesh::ElementIndices;
use crate::rendering::shader;
use crate::rendering::shader::{BoundShader, ShaderParamInfo, ShaderType};

#[derive(Clone, Debug)]
pub enum ShaderCreationError {
    FailedToCreateShader,
    FailedToCompile(String),
}

impl Display for ShaderCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShaderCreationError::FailedToCreateShader => write!(f, "Failed to create shader"),
            ShaderCreationError::FailedToCompile(s) => write!(f, "Failed to compile shader: {}", s),
        }
    }
}

impl Error for ShaderCreationError {}

#[derive(Debug)]
pub struct Shader {
    context: Rc<WebGlRenderingContext>,
    shader: WebGlShader,
    shader_type: ShaderType,
}

impl Shader {
    pub fn compile(
        context: Rc<WebGlRenderingContext>,
        shader_type: ShaderType,
        source: &str,
    ) -> Result<Shader, ShaderCreationError> {
        let shader = context
            .create_shader(to_gl_shader_type(shader_type))
            .ok_or(ShaderCreationError::FailedToCreateShader)?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context
            .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(Shader {
                context,
                shader,
                shader_type,
            })
        } else {
            Err(context
                .get_shader_info_log(&shader)
                .map(ShaderCreationError::FailedToCompile)
                .unwrap_or_else(|| {
                    ShaderCreationError::FailedToCompile(String::from("Unknown compilation error"))
                }))
        }
    }

    pub fn is_same_context(&self, context: &Rc<WebGlRenderingContext>) -> bool {
        Rc::ptr_eq(&self.context, context)
    }
}

impl shader::Shader for Shader {
    type RenderingContext = WebGlContext;

    fn shader_type(&self) -> ShaderType {
        self.shader_type
    }
}

#[derive(Debug)]
struct ShaderUniformInformation {
    name: Box<str>,
    location: WebGlUniformLocation,
}

#[derive(Clone, Debug)]
pub enum ShaderLinkError {
    FailedToCreateShaderProgram,
    FailedToLink(String),
}

impl Display for ShaderLinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShaderLinkError::FailedToCreateShaderProgram => {
                write!(f, "Failed to create shader program")
            }
            ShaderLinkError::FailedToLink(s) => write!(f, "Failed to link shader: {}", s),
        }
    }
}

impl Error for ShaderLinkError {}

#[derive(Debug)]
pub struct ShaderProgram {
    program: WebGlProgram,
    context: Rc<WebGlRenderingContext>,
    attributes: HashMap<Box<str>, ShaderParamInfo>,
    uniforms: Vec<ShaderUniformInformation>,
}

impl ShaderProgram {
    pub fn link<'a, T: Iterator<Item = &'a Shader>>(
        context: Rc<WebGlRenderingContext>,
        shaders: T,
    ) -> Result<ShaderProgram, ShaderLinkError> {
        let program = context
            .create_program()
            .ok_or(ShaderLinkError::FailedToCreateShaderProgram)?;
        for shader in shaders {
            context.attach_shader(&program, &shader.shader)
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
                .map(ShaderLinkError::FailedToLink)
                .unwrap_or_else(|| {
                    ShaderLinkError::FailedToLink(String::from("Unknown linking error"))
                }))
        }
    }

    pub fn is_same_context(&self, context: &Rc<WebGlRenderingContext>) -> bool {
        Rc::ptr_eq(&self.context, context)
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

pub struct WebGlBoundShader {
    context: Rc<WebGlRenderingContext>,
    shader: Rc<ShaderProgram>,
}

impl WebGlBoundShader {
    pub fn new(context: Rc<WebGlRenderingContext>, shader: Rc<ShaderProgram>) -> WebGlBoundShader {
        WebGlBoundShader { context, shader }
    }
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

fn to_gl_shader_type(shader_type: ShaderType) -> u32 {
    match shader_type {
        ShaderType::Vertex => WebGlRenderingContext::VERTEX_SHADER,
        ShaderType::Fragment => WebGlRenderingContext::FRAGMENT_SHADER,
    }
}

fn to_gl_element_type(attr_type: IndexType) -> u32 {
    match attr_type {
        IndexType::UnsignedByte => WebGlRenderingContext::UNSIGNED_BYTE,
        IndexType::UnsignedShort => WebGlRenderingContext::UNSIGNED_SHORT,
        IndexType::UnsignedInt => WebGlRenderingContext::UNSIGNED_INT,
    }
}
