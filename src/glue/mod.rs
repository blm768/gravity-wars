use std::cell::RefCell;
use std::collections::VecDeque;

use std::io::Cursor;
use std::rc::Rc;
use std::str;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::{Element, HtmlCanvasElement};
use web_sys::{WebGlRenderingContext, WebGlShader};

use gltf::Gltf;

use glue::asset::{AssetData, AssetLoader, FetchError};
use glue::callback::{AnimationFrameCallback, IntervalCallback};
use glue::game_handle::GameHandle;
use glue::webgl::game_renderer::WebGlRenderer;
use glue::webgl::{ShaderType, WebGlContext};
use rendering::material::MaterialShader;
use rendering::mesh::gltf::GltfLoader;
use state::event::InputEvent;
use state::mapgen;
use state::GameState;
use state_renderer::{GameRenderer, MeshRenderer};

pub mod asset;
pub mod callback;
pub mod game_handle;
pub mod webgl;

/// Game state update interval (in milliseconds)
pub const STATE_UPDATE_INTERVAL: usize = 33;

#[wasm_bindgen]
extern "C" {
    // TODO: just use web-sys?
    #[wasm_bindgen(js_namespace=console)]
    pub fn log(text: &str);
}

pub fn get_canvas() -> Option<(Element, HtmlCanvasElement)> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas_element = document.get_element_by_id("game_canvas")?;
    let canvas = document
        .get_element_by_id("game_canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    Some((canvas_element, canvas))
}

pub fn get_webgl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, String> {
    canvas
        .get_context("webgl")
        .map_err(|_| String::from("Error retrieving context"))?
        .ok_or_else(|| String::from("Context is null"))?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|obj| obj.to_string().into())
}

#[wasm_bindgen]
pub fn load_assets() -> AssetLoader {
    let assets = AssetLoader::new();
    assets.load("shaders/vertex.glsl");
    assets.load("shaders/fragment.glsl");
    assets.load("assets/meshes/ship.glb");

    assets
}

fn compile_shader_from_asset(
    url: &str,
    asset: Result<&[u8], FetchError>,
    context: &WebGlRenderingContext,
    shader_type: ShaderType,
) -> Result<WebGlShader, String> {
    match asset {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            webgl::compile_shader(context, shader_type, text)
        }
        Err(error) => Err(format!("Unable to load asset {}: {}", url, error)),
    }
}

#[wasm_bindgen]
pub fn start_game(assets: &AssetData) -> JsValue {
    match try_start_game(assets) {
        Ok(handle) => JsValue::from(handle),
        Err(err) => {
            log(&format!("Error starting game: {}", err));
            JsValue::NULL
        }
    }
}

fn try_start_game(assets: &AssetData) -> Result<GameHandle, String> {
    let (canvas_element, canvas) =
        get_canvas().ok_or_else(|| String::from("Unable to find canvas"))?;
    let gl_context = get_webgl_context(&canvas)?;
    let context = Rc::new(WebGlContext::new(canvas_element, canvas, gl_context));

    let mut state = GameState::new();
    mapgen::generate_map(&mut state);

    let vertex_shader = compile_shader_from_asset(
        "shaders/vertex.glsl",
        assets.get("shaders/vertex.glsl"),
        context.gl_context(),
        ShaderType::Vertex,
    )?;
    let fragment_shader = compile_shader_from_asset(
        "shaders/fragment.glsl",
        assets.get("shaders/fragment.glsl"),
        context.gl_context(),
        ShaderType::Fragment,
    )?;
    let program = webgl::ShaderProgram::link(
        context.gl_context().clone(),
        [vertex_shader, fragment_shader].iter(),
    )?;
    let mat_shader = MaterialShader::new(Box::new(program)).map_err(|e| format!("{:?}", e))?;

    let renderer = Rc::new(WebGlRenderer::new(Rc::clone(&context), mat_shader));
    renderer.configure_context();

    let raw_gltf = assets
        .get("assets/meshes/ship.glb")
        .map_err(|_| String::from("Unable to retrieve mesh asset"))?;
    let gltf = Gltf::from_reader(Cursor::new(raw_gltf)).map_err(|e| format!("{:?}", e))?;
    let mut loader = GltfLoader::new(Rc::clone(renderer.context()), &gltf);
    let first_mesh = gltf
        .meshes()
        .next()
        .ok_or_else(|| String::from("Unable to find mesh"))?;
    let mesh = loader
        .load_mesh(&first_mesh)
        .map_err(|_| String::from("Unable to load mesh"))?;
    let ship_renderer = Rc::new(MeshRenderer::new(
        Rc::clone(&renderer) as Rc<GameRenderer<Context = WebGlContext>>,
        mesh,
    ));

    mapgen::add_ships(&mut state, ship_renderer);
    mapgen::add_planets(
        &mut state,
        Rc::clone(&renderer) as Rc<GameRenderer<Context = WebGlContext>>,
    )
    .map_err(|_| String::from("Unable to create planet mesh/renderer"))?;

    let shared_state = Rc::new(RefCell::new(state));

    let render_state = Rc::clone(&shared_state);
    let renderer_clone = Rc::clone(&renderer);
    let render_frame = move |_milliseconds: f64| {
        renderer_clone
            .render(&render_state.borrow())
            .unwrap_or_else(|e| log(&e.to_string()));
    };

    let input_queue = Rc::new(RefCell::new(VecDeque::<InputEvent>::new()));

    let update_state = Rc::clone(&shared_state);
    let update_input_queue = Rc::clone(&input_queue);
    let update_game = move || {
        let mut queue = update_input_queue.borrow_mut();
        let queue_len = queue.len();
        for event in queue.drain(0..queue_len) {
            if let Err(e) = update_state.borrow_mut().handle_input(&event) {
                log(&e.to_string());
            }
        }
        update_state.borrow_mut().update_missiles();
    };

    let mut render_callback = AnimationFrameCallback::new(render_frame);
    render_callback.start()?;
    let mut update_callback = IntervalCallback::new(update_game, STATE_UPDATE_INTERVAL as i32);
    update_callback.start()?;

    Ok(GameHandle::new(
        shared_state,
        renderer,
        input_queue,
        render_callback,
        update_callback,
    ))
}
