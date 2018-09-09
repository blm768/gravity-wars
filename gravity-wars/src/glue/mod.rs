use wasm_bindgen::prelude::*;

use std::str;

use cgmath::{Matrix4, SquareMatrix, Vector3};
use web_sys::{WebGlRenderingContext, WebGlShader};

use glue::asset::{AssetData, AssetLoader, FetchError};
use glue::webgl::{Buffer, BufferBinding, ShaderType, VertexAttributeBinding, WebGlRenderer};
use rendering::renderer::GameRenderer;
use rendering::shader::{MaterialShaderInfo, ShaderProgram};
use state::GameState;

pub mod asset;
pub mod webgl;

#[wasm_bindgen]
extern "C" {
    // TODO: just use web-sys?
    #[wasm_bindgen(js_namespace=console)]
    pub fn log(text: &str);
}

#[wasm_bindgen(module = "./glue")]
extern "C" {
    // TODO: handle this more elegantly.
    #[wasm_bindgen]
    pub fn getWebGlContext() -> WebGlRenderingContext;
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
) -> Option<WebGlShader> {
    match asset {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            let compiled = webgl::compile_shader(renderer.context(), shader_type, text);
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

    let renderer = WebGlRenderer::new(getWebGlContext());
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
    let program = webgl::ShaderProgram::link(
        renderer.context().clone(),
        [vertex_shader, fragment_shader].iter(),
    ).unwrap();
    let info = MaterialShaderInfo::from_program(&program).unwrap();

    let mut vertices = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    ];

    log("Shaders compiled");

    match Buffer::new(renderer.context().clone(), BufferBinding::ArrayBuffer) {
        Some(buf) => {
            buf.set_data(&mut vertices);

            log("Buffers created");

            let position_binding = VertexAttributeBinding::typed::<Vector3<f32>>();
            buf.bind_to_attribute(info.position.index, &position_binding);

            log("Bound to attribute");
        }
        None => log("Unable to create buffer"),
    }
    program.activate();

    renderer.context().clear_color(0.0, 0.0, 0.0, 1.0);
    renderer
        .context()
        .clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    let projection: Matrix4<f32> = state.camera.projection(renderer.aspect_ratio()).into();
    let modelview = Matrix4::<f32>::identity();

    program.set_uniform_mat4(info.projection.index, projection);
    program.set_uniform_mat4(info.model_view.index, modelview);

    log("Uniforms bound");
}
