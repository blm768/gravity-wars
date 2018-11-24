use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use cgmath::Vector2;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use glue::callback::{AnimationFrameCallback, IntervalCallback};
use glue::webgl::game_renderer::WebGlRenderer;
use state::event::{InputEvent, MissileParams};
use state::GameState;

// Main interface between JavaScript and Rust
// Also keeps game callbacks alive so the closures don't get dropped.
#[wasm_bindgen]
pub struct GameHandle {
    game_state: Rc<RefCell<GameState>>,
    renderer: Rc<WebGlRenderer>,
    input_queue: Rc<RefCell<VecDeque<InputEvent>>>,
    _render_callback: AnimationFrameCallback,
    _update_callback: IntervalCallback,
}

impl GameHandle {
    pub fn new(
        game_state: Rc<RefCell<GameState>>,
        renderer: Rc<WebGlRenderer>,
        input_queue: Rc<RefCell<VecDeque<InputEvent>>>,
        render_callback: AnimationFrameCallback,
        update_callback: IntervalCallback,
    ) -> GameHandle {
        GameHandle {
            game_state,
            renderer,
            input_queue,
            _render_callback: render_callback,
            _update_callback: update_callback,
        }
    }
}

#[wasm_bindgen]
impl GameHandle {
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.renderer.context().canvas().clone()
    }

    /// Registers a camera pan event with the given X and Y deltas (in pixels)
    #[wasm_bindgen(js_name = onPan)]
    pub fn on_pan(&mut self, x: f32, y: f32) {
        let projection = self
            .game_state
            .borrow()
            .camera()
            .projection(self.renderer.context().aspect_ratio());
        let pan_factor =
            (projection.top - projection.bottom) / self.renderer.context().height() as f32;
        let delta = Vector2::new(x * pan_factor, y * pan_factor);
        self.input_queue
            .borrow_mut()
            .push_back(InputEvent::PanCamera(delta));
    }

    #[wasm_bindgen(js_name = onZoom)]
    pub fn on_zoom(&mut self, factor: f32) {
        self.input_queue
            .borrow_mut()
            .push_back(InputEvent::ZoomCamera(factor));
    }

    #[wasm_bindgen(js_name = onFire)]
    pub fn on_fire(&mut self, angle: f32) {
        self.input_queue
            .borrow_mut()
            .push_back(InputEvent::FireMissile(MissileParams {
                angle,
                speed: 10.0,
            }));
    }
}
