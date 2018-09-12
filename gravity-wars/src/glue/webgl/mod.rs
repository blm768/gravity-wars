use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use cgmath::Matrix4;
use web_sys::{Element, HtmlCanvasElement};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation};

use rendering::renderer::GameRenderer;
use rendering::shader;
use rendering::shader::ShaderParamInfo;
use state::GameState;

pub mod buffer;
pub mod mesh;

pub const DEPTH_BUFFER_BIT: i32 = 0x0100;
pub const STENCIL_BUFFER_BIT: i32 = 0x0400;
pub const COLOR_BUFFER_BIT: i32 = 0x4000;

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
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

struct ShaderUniformInformation {
    name: Box<str>,
    location: WebGlUniformLocation,
}

pub struct ShaderProgram {
    program: WebGlProgram,
    context: Rc<WebGlRenderingContext>,
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
            let uniforms = ShaderProgram::get_uniform_info(&context, &program);
            Ok(ShaderProgram {
                program,
                context,
                uniforms,
            })
        } else {
            Err(context
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown error creating program object".into()))
        }
    }

    fn get_uniform_info(
        context: &WebGlRenderingContext,
        program: &WebGlProgram,
    ) -> Vec<ShaderUniformInformation> {
        let num_uniforms = context
            .get_program_parameter(&program, WebGlRenderingContext::ACTIVE_UNIFORMS)
            .as_f64()
            .unwrap() as u32; // TODO: create (and use) a safe conversion helper...
        let mut uniforms = Vec::<ShaderUniformInformation>::new();
        uniforms.reserve_exact(num_uniforms as usize);

        for i in 0..num_uniforms {
            // TODO: log errors?
            if let Some(info) = context.get_active_uniform(&program, i) {
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
    fn attributes(&self) -> HashMap<Box<str>, ShaderParamInfo> {
        let num_attributes = self
            .context
            .get_program_parameter(&self.program, WebGlRenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .unwrap() as u32;
        let mut attributes = HashMap::<Box<str>, ShaderParamInfo>::new();
        for i in 0..num_attributes {
            if let Some(info) = self.context.get_active_attrib(&self.program, i) {
                // TODO: log errors?
                attributes.insert(
                    info.name().into_boxed_str(),
                    ShaderParamInfo { index: i as usize },
                );
            }
        }
        attributes
    }

    fn uniforms(&self) -> HashMap<Box<str>, ShaderParamInfo> {
        self.uniforms
            .iter()
            .enumerate()
            .map({ |(i, uniform)| (uniform.name.clone(), ShaderParamInfo { index: i }) })
            .collect::<HashMap<_, _>>()
    }

    fn activate(&self) {
        self.context.use_program(Some(&self.program));
    }

    fn set_uniform_mat4(&self, index: usize, value: Matrix4<f32>) {
        let raw: &[f32; 16] = value.as_ref();
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.uniforms[index].location),
            false,
            raw,
        );
    }
}

pub struct WebGlRenderer {
    canvas_element: Element,
    canvas: HtmlCanvasElement,
    context: Rc<WebGlRenderingContext>,
}

impl WebGlRenderer {
    pub fn new(
        canvas_element: Element,
        canvas: HtmlCanvasElement,
        context: WebGlRenderingContext,
    ) -> WebGlRenderer {
        WebGlRenderer {
            canvas_element,
            canvas,
            context: Rc::new(context),
        }
    }

    pub fn context(&self) -> &Rc<WebGlRenderingContext> {
        &self.context
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.context.drawing_buffer_width() as f32 / self.context.drawing_buffer_height() as f32
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

        self.context.viewport(
            0,
            0,
            self.context.drawing_buffer_width(),
            self.context.drawing_buffer_height(),
        );
    }
}

impl GameRenderer for WebGlRenderer {
    fn render(&self, state: &GameState) -> Result<(), Box<Error>> {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let projection: Matrix4<f32> = state.camera.projection(self.aspect_ratio()).into();

        // TODO: implement.
        for entity in &state.map.entities {
            let modelview_transform = Matrix4::<f32>::from_translation(entity.position.extend(0.0));
        }

        Ok(())
    }
}
