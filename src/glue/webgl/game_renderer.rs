use std::error::Error;
use std::rc::Rc;

use web_sys::WebGlRenderingContext;

use glue::webgl::WebGlContext;
use state::GameState;
use state_renderer::GameRenderer;

pub struct WebGlRenderer {
    context: Rc<WebGlContext>,
}

impl WebGlRenderer {
    pub fn new(context: WebGlContext) -> WebGlRenderer {
        WebGlRenderer {
            context: Rc::new(context),
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
    fn render(&self, _state: &GameState) -> Result<(), Box<Error>> {
        self.gl_context().clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl_context()
            .clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        Ok(())
    }
}
