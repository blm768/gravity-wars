use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use web_sys::WebGlRenderingContext;
use web_sys::{Element, HtmlCanvasElement};

use crate::glue::webgl::shader::{ShaderProgram, WebGlBoundShader};
use crate::rendering::context::RenderingContext;
use crate::rendering::shader::{ShaderBindError, ShaderType};

pub mod buffer;
pub mod game_renderer;
pub mod shader;
pub mod texture;

#[derive(Debug)]
pub struct WebGlContext {
    canvas_element: Element,
    canvas: HtmlCanvasElement,
    gl_context: Rc<WebGlRenderingContext>,
    shader_bound: Cell<bool>,
}

static HAS_WARNED_ABOUT_PIXEL_RATIO: AtomicBool = AtomicBool::new(false);

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

    /**
     * Returns the device pixel ratio.
     *
     * Always returns a finite, positive value.
     */
    pub fn device_pixel_ratio(&self) -> f64 {
        let ratio = web_sys::window().unwrap().device_pixel_ratio();
        if !ratio.is_finite() || ratio <= 0.0 {
            if !HAS_WARNED_ABOUT_PIXEL_RATIO.swap(true, Ordering::Relaxed) {
                log::warn!("Invalid device pixel ratio");
            }
            1.0
        } else {
            ratio
        }
    }

    pub fn set_viewport(&self) {
        use std::cmp::max;

        let scale = self.device_pixel_ratio();
        let width = (max(self.canvas_element.client_width(), 0) as f64 * scale) as u32;
        let height = (max(self.canvas_element.client_height(), 0) as f64 * scale) as u32;

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

impl RenderingContext for WebGlContext {
    type AttributeBuffer = buffer::AttributeBuffer;
    type IndexBuffer = buffer::IndexBuffer;
    type Shader = shader::Shader;
    type ShaderCreationError = shader::ShaderCreationError;
    type ShaderProgram = ShaderProgram;
    type ShaderLinkError = shader::ShaderLinkError;
    type BoundShader = shader::WebGlBoundShader;
    type Texture = texture::Texture;

    fn make_attribute_buffer(&self) -> Result<Self::AttributeBuffer, ()> {
        Self::AttributeBuffer::new(Rc::clone(&self.gl_context)).ok_or(())
    }

    fn make_index_buffer(&self) -> Result<Self::IndexBuffer, ()> {
        Self::IndexBuffer::new(Rc::clone(&self.gl_context)).ok_or(())
    }

    fn make_texture(&self) -> Result<Self::Texture, ()> {
        Self::Texture::new(Rc::clone(&self.gl_context)).ok_or(())
    }

    fn compile_shader(
        &self,
        shader_type: ShaderType,
        source: &str,
    ) -> Result<Self::Shader, Self::ShaderCreationError> {
        Self::Shader::compile(Rc::clone(&self.gl_context), shader_type, source)
    }

    fn link_shader_program<'a, T: Iterator<Item = &'a Self::Shader>>(
        &self,
        shaders: T,
    ) -> Result<Self::ShaderProgram, Self::ShaderLinkError> {
        Self::ShaderProgram::link(Rc::clone(&self.gl_context), shaders)
    }

    fn bind_shader(
        &self,
        shader: Rc<Self::ShaderProgram>,
    ) -> Result<Self::BoundShader, ShaderBindError> {
        if self.shader_bound.get() {
            return Err(ShaderBindError::CannotBindMoreShaders);
        }
        if !shader.is_same_context(&self.gl_context) {
            return Err(ShaderBindError::InvalidContextForShader);
        }

        shader.bind();
        Ok(WebGlBoundShader::new(Rc::clone(&self.gl_context), shader))
    }
}
