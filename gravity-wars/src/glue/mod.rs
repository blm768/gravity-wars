use wasm_bindgen::prelude::*;

use std::str;

use glue::asset::{AssetData, AssetLoader, FetchError};
use glue::webgl::{ShaderProgram, ShaderType, WebGLShader, WebGlRenderer};
use rendering::renderer::GameRenderer;
use rendering::shader::MaterialShaderInfo;
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

#[wasm_bindgen]
pub fn init_game() -> AssetLoader {
    let assets = AssetLoader::new(start_game);
    assets.load("shaders/vertex.glsl");
    assets.load("shaders/fragment.glsl");

    assets
}

fn compile_shader_from_asset(
    asset: Result<&[u8], FetchError>,
    renderer: &WebGlRenderer,
    shader_type: ShaderType,
) -> Option<WebGLShader> {
    match asset {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            let compiled = WebGLShader::compile(renderer.context(), shader_type, text);
            match compiled {
                Ok(shader) => Some(shader),
                Err(ref error) => {
                    log(error);
                    None
                }
            }
        }
        Err(error) => {
            log(&format!("Unable to load asset {}", "shaders/vertex.glsl"));
            log(&format!("{}", error));
            None
        }
    }
}

#[wasm_bindgen]
pub fn start_game(assets: &AssetData) {
    let state = GameState::new();

    let renderer = WebGlRenderer::new();
    if let Err(_error) = renderer.render(&state) {
        log("Rendering error");
    }

    let vertex_shader = compile_shader_from_asset(
        assets.get("shaders/vertex.glsl"),
        &renderer,
        ShaderType::Vertex,
    ).unwrap();
    let fragment_shader = compile_shader_from_asset(
        assets.get("shaders/fragment.glsl"),
        &renderer,
        ShaderType::Fragment,
    ).unwrap();
    let program = ShaderProgram::link(
        renderer.context().clone(),
        [vertex_shader, fragment_shader].iter(),
    ).unwrap();
    let info = MaterialShaderInfo::from_program(&program);
    log(&format!("{:?}", info));

    log("Shaders compiled");
}
