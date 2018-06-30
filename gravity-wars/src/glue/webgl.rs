use wasm_bindgen::prelude::*;

pub const DEPTH_BUFFER_BIT: i32 = 0x0100;
pub const STENCIL_BUFFER_BIT: i32 = 0x0400;
pub const COLOR_BUFFER_BIT: i32 = 0x4000;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace=glue, js_name=getWebGLContext)]
    pub fn get_webgl_context() -> WebGLRenderingContext;

    pub type WebGLRenderingContext;
    #[wasm_bindgen(method, js_name=clearColor)]
    pub fn clear_color(context: &WebGLRenderingContext, r: f32, g: f32, b: f32, a: f32);
    #[wasm_bindgen(method)]
    pub fn clear(context: &WebGLRenderingContext, mask: i32);
}

impl WebGLRenderingContext {
    pub fn new() -> WebGLRenderingContext {
        get_webgl_context()
    }
}
