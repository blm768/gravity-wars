use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::glue::callback::Callback;
use crate::glue::webgl::game_renderer::WebGlRenderer;
use crate::state::event::{InputEvent, MissileParams};
use crate::state::GameState;

/// Main interface between JavaScript and Rust
///
/// Also keeps game callbacks alive so the closures don't get dropped.
#[wasm_bindgen]
pub struct GameHandle {
    game_state: Rc<RefCell<GameState>>,
    renderer: Rc<WebGlRenderer>,
    input_queue: Rc<RefCell<VecDeque<InputEvent>>>,
    interface: Rc<RefCell<Option<GameInterface>>>,
    callbacks: Vec<Box<dyn Callback>>,
}

impl GameHandle {
    pub fn new(game_state: Rc<RefCell<GameState>>, renderer: Rc<WebGlRenderer>) -> GameHandle {
        GameHandle {
            game_state,
            renderer,
            input_queue: Rc::new(RefCell::new(VecDeque::new())),
            interface: Rc::new(RefCell::new(None)),
            callbacks: Vec::new(),
        }
    }

    pub fn game_state(&self) -> &Rc<RefCell<GameState>> {
        &self.game_state
    }

    pub fn input_queue(&self) -> &Rc<RefCell<VecDeque<InputEvent>>> {
        &self.input_queue
    }

    pub fn interface(&self) -> &Rc<RefCell<Option<GameInterface>>> {
        &self.interface
    }

    /// Adds a callback to the internal list so it won't get dropped while the handle exists
    pub fn add_callback(&mut self, callback: Box<dyn Callback>) {
        self.callbacks.push(callback);
    }
}

#[wasm_bindgen]
impl GameHandle {
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.renderer.context().canvas().clone()
    }

    #[wasm_bindgen(js_name = hasActiveMissiles)]
    pub fn has_active_missiles(&self) -> bool {
        self.game_state
            .borrow()
            .iter_entities()
            .any(|e| match e.missile_trail {
                Some(ref trail) => trail.time_to_live > 0.0,
                None => false,
            })
    }

    #[wasm_bindgen(js_name = currentPlayer)]
    pub fn current_player(&self) -> Option<u32> {
        self.game_state.borrow().current_player().map(|p| p as u32)
    }

    #[wasm_bindgen(js_name = currentPlayerColor)]
    pub fn current_player_color(&self) -> Option<Vec<u8>> {
        use rgb::{ComponentMap, ComponentSlice};
        let state = self.game_state.borrow();
        let color = state.current_player().map(|p| {
            state.players()[p]
                .color
                .map(|c| (c * 255.0).max(0.0).min(255.0) as u8)
        })?;
        Some(color.as_slice().to_vec())
    }

    /// Called by the JavaScript glue code when the game interface has been initialized
    #[wasm_bindgen(js_name = onInterfaceReady)]
    pub fn on_interface_ready(&self, game_interface: GameInterface) {
        *self.interface.borrow_mut() = Some(game_interface);
    }

    /// Registers a camera pan event with the given X and Y deltas (in pixels)
    #[wasm_bindgen(js_name = onPan)]
    pub fn on_pan(&mut self, x: f32, y: f32) {
        let camera = &mut self.game_state.borrow_mut().camera;
        camera.aspect_ratio = self.renderer.context().aspect_ratio();
        let projection = camera.projection();
        let pan_factor =
            (projection.top() - projection.bottom()) / self.renderer.context().height() as f32;
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
    pub fn on_fire(&mut self, angle: f32, speed: f32) {
        self.input_queue
            .borrow_mut()
            .push_back(InputEvent::FireMissile(MissileParams { angle, speed }));
    }
}

#[wasm_bindgen(module = "/src/glue/game_interface.js")]
extern "C" {
    /// Handle to the game's UI controls
    pub type GameInterface;

    #[wasm_bindgen(constructor)]
    pub fn new() -> GameInterface;

    #[wasm_bindgen(method, js_name = "updateUI")]
    pub fn update_ui(interface: &GameInterface);
}

#[wasm_bindgen(js_name = "initInterface")]
pub fn init_interface() -> GameInterface {
    GameInterface::new()
}
