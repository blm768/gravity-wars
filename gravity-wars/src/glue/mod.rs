use wasm_bindgen::prelude::*;

use std::str;

use glue::asset::{AssetData, AssetLoader};
use renderer::{GameRenderer, WebGlRenderer};
use state::GameState;

pub mod asset;
pub mod webgl;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    pub fn log(text: &str);

    #[wasm_bindgen(js_namespace=glue, js_name=getElementById)]
    pub fn get_element_by_id(id: &str) -> Element;

    #[wasm_bindgen(js_namespace = glue, js_name=isNull)]
    pub fn is_null(element: &Element) -> Element;

    pub type Element;
    #[wasm_bindgen(method, js_name=toString)]
    pub fn to_string(element: &Element) -> String;
}

pub struct InputBox {}

pub struct GameControls {
    angle: InputBox,
    velocity: InputBox,
}

#[wasm_bindgen]
pub fn init_game() -> AssetLoader {
    let controls = get_element_by_id("game_controls");

    let assets = AssetLoader::new(start_game);
    assets.load("shaders/vertex.glsl");

    let state = GameState::new();

    let renderer = WebGlRenderer::new();
    if let Err(_error) = renderer.render(&state) {
        log("Rendering error");
    }

    assets
}

#[wasm_bindgen]
pub fn start_game(assets: AssetData) {
    let AssetData(data) = assets;
    match data.get("shaders/vertex.glsl") {
        Some(Ok(ref data)) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            log(text);
        }
        _ => {
            log("Missing asset");
        }
    }
}
