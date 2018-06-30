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
    let assets = AssetLoader::new(start_game);
    assets.load("shaders/vertex.glsl");

    assets
}

#[wasm_bindgen]
pub fn start_game(assets: AssetData) {
    let state = GameState::new();

    let renderer = WebGlRenderer::new();
    if let Err(_error) = renderer.render(&state) {
        log("Rendering error");
    }

    match assets.get("shaders/vertex.glsl") {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            let compiled = webgl::compile_shader(renderer.context(), webgl::VERTEX_SHADER, text);
            match compiled {
                Ok(_) => log("Shader compiled"),
                Err(ref error) => log(error),
            }
        }
        err => {
            log(&format!("Unable to load asset {}", "shaders/vertex.glsl"));
        }
    }
}
