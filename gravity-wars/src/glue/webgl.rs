use wasm_bindgen::prelude::*;

use std::error::Error;

use cgmath::Matrix4;

use rendering::renderer::GameRenderer;
use state::GameState;

pub const DEPTH_BUFFER_BIT: i32 = 0x0100;
pub const STENCIL_BUFFER_BIT: i32 = 0x0400;
pub const COLOR_BUFFER_BIT: i32 = 0x4000;

pub const VERTEX_SHADER: i32 = 0x8B31;
pub const FRAGMENT_SHADER: i32 = 0x8B30;
pub const COMPILE_STATUS: i32 = 0x8B81;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=glue, js_name=getWebGLContext)]
    pub fn get_webgl_context() -> WebGLRenderingContext;

    pub type WebGLRenderingContext;
    #[wasm_bindgen(method, js_name=clearColor)]
    pub fn clear_color(context: &WebGLRenderingContext, r: f32, g: f32, b: f32, a: f32);
    #[wasm_bindgen(method)]
    pub fn clear(context: &WebGLRenderingContext, mask: i32);

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
}

impl WebGLRenderingContext {
    pub fn new() -> WebGLRenderingContext {
        get_webgl_context()
    }
}

pub fn compile_shader(
    context: &WebGLRenderingContext,
    shader_type: i32,
    source: &str,
) -> Result<WebGLShader, String> {
    let shader = context.create_shader(shader_type);
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context.get_shader_parameter_boolean(&shader, COMPILE_STATUS) {
        Ok(shader)
    } else {
        Err(context.get_shader_info_log(&shader))
    }
}

pub struct WebGlRenderer {
    context: WebGLRenderingContext,
    aspect_ratio: f32,
}

impl WebGlRenderer {
    pub fn new() -> WebGlRenderer {
        let context = WebGLRenderingContext::new();
        WebGlRenderer {
            context,
            aspect_ratio: 1.0,
        }
    }

    pub fn context(&self) -> &WebGLRenderingContext {
        &self.context
    }
}

impl GameRenderer for WebGlRenderer {
    fn render(&self, state: &GameState) -> Result<(), Box<Error>> {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(COLOR_BUFFER_BIT);

        let projection: Matrix4<f32> = state.camera.projection(self.aspect_ratio).into();

        // TODO: implement.
        for entity in &state.map.entities {}

        Ok(())
    }
}
