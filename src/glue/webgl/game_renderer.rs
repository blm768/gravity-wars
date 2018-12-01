use std::error::Error;
use std::rc::Rc;

use web_sys::WebGlRenderingContext;

use crate::glue::webgl::WebGlContext;
use crate::rendering::line::LineShader;
use crate::rendering::material::MaterialShader;
use crate::state::GameState;
use crate::state_renderer::GameRenderer;

#[derive(Debug)]
pub struct WebGlRenderer {
    context: Rc<WebGlContext>,
    material_shader: MaterialShader<WebGlContext>,
    line_shader: LineShader<WebGlContext>,
}

impl WebGlRenderer {
    pub fn new(
        context: Rc<WebGlContext>,
        material_shader: MaterialShader<WebGlContext>,
        line_shader: LineShader<WebGlContext>,
    ) -> WebGlRenderer {
        WebGlRenderer {
            context,
            material_shader,
            line_shader,
        }
    }

    pub fn configure_context(&self) {
        self.gl_context().enable(WebGlRenderingContext::CULL_FACE);
        self.gl_context().cull_face(WebGlRenderingContext::BACK);
        self.gl_context().enable(WebGlRenderingContext::DEPTH_TEST);
        self.gl_context().depth_func(WebGlRenderingContext::LESS);
    }

    pub fn context(&self) -> &Rc<WebGlContext> {
        &self.context
    }

    pub fn gl_context(&self) -> &Rc<WebGlRenderingContext> {
        self.context.gl_context()
    }
}

impl GameRenderer for WebGlRenderer {
    type Context = WebGlContext;

    fn context(&self) -> &Self::Context {
        &self.context
    }

    fn material_shader(&self) -> &MaterialShader<WebGlContext> {
        &self.material_shader
    }

    fn line_shader(&self) -> &LineShader<WebGlContext> {
        &self.line_shader
    }

    fn render(&self, state: &mut GameState) -> Result<(), Box<Error>> {
        self.context.set_viewport();
        state.camera.aspect_ratio = self.context.aspect_ratio();
        self.gl_context().clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl_context().clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        for entity in state.iter_entities() {
            if let Some(ref renderer) = entity.renderer {
                renderer.render(entity, state);
            }
        }

        Ok(())
    }
}
