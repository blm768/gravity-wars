use std::error::Error;

use glue::webgl;
use glue::webgl::WebGLRenderingContext;

use state::GameState;

pub trait GameRenderer {
    fn render(&self, &GameState) -> Result<(), Box<Error>>;
}

pub struct WebGlRenderer {
    context: WebGLRenderingContext,
}

impl WebGlRenderer {
    pub fn new() -> WebGlRenderer {
        let context = WebGLRenderingContext::new();
        WebGlRenderer { context }
    }
}

impl GameRenderer for WebGlRenderer {
    fn render(&self, state: &GameState) -> Result<(), Box<Error>> {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(webgl::COLOR_BUFFER_BIT);
        // TODO: implement.
        Ok(())
    }
}
