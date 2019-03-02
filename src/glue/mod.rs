use std::cell::RefCell;

use std::io::Cursor;
use std::panic;
use std::rc::Rc;
use std::str;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::WebGlRenderingContext;
use web_sys::{Element, HtmlCanvasElement};

use gltf::Gltf;

use crate::glue::asset::{AssetData, AssetLoader};
use crate::glue::callback::{AnimationFrameCallback, IntervalCallback};
use crate::glue::game_handle::GameHandle;
use crate::glue::webgl::game_renderer::WebGlRenderer;
use crate::glue::webgl::shader::{Shader, ShaderProgram};
use crate::glue::webgl::WebGlContext;
use crate::rendering::context::RenderingContext;
use crate::rendering::line::LineShader;
use crate::rendering::material::MaterialShader;
use crate::rendering::mesh::gltf::GltfLoader;
use crate::rendering::shader::ShaderType;
use crate::rendering::Rgb;
use crate::state::mapgen::{self, MapgenParams};
use crate::state::{EntityRenderer, GameState, Player};
use crate::state_renderer::{GameRenderer, MissileTrailRenderer};

pub mod asset;
pub mod callback;
pub mod game_handle;
pub mod webgl;

const DEFAULT_MAP_WIDTH: f32 = 150.0;
const DEFAULT_MAP_HEIGHT: f32 = 100.0;

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
    assets.load("shaders/line_vertex.glsl");
    assets.load("shaders/line_fragment.glsl");
    assets.load("assets/meshes/ship.glb");

    assets
}

fn compile_shader_from_asset(
    url: &str,
    assets: &AssetData,
    context: &WebGlContext,
    shader_type: ShaderType,
) -> Result<Shader, String> {
    match assets.get(url) {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            context
                .compile_shader(shader_type, text)
                .map_err(|e| format!("{}: {}", url, e))
        }
        Err(error) => Err(format!("Unable to load asset {}: {}", url, error)),
    }
}

fn load_program_from_assets(
    vert_url: &str,
    frag_url: &str,
    assets: &AssetData,
    context: &WebGlContext,
) -> Result<ShaderProgram, String> {
    let vertex_shader = compile_shader_from_asset(vert_url, assets, context, ShaderType::Vertex)?;
    let fragment_shader =
        compile_shader_from_asset(frag_url, assets, context, ShaderType::Fragment)?;
    context
        .link_shader_program([vertex_shader, fragment_shader].iter())
        .map_err(|e| format!("{}", e))
}

#[wasm_bindgen]
pub fn start_game(assets: &AssetData) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook)); // TODO: make this happen earlier.
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

    let mat_program = load_program_from_assets(
        "shaders/vertex.glsl",
        "shaders/fragment.glsl",
        assets,
        &context,
    )?;
    let line_program = load_program_from_assets(
        "shaders/line_vertex.glsl",
        "shaders/line_fragment.glsl",
        assets,
        &context,
    )?;
    let mat_shader = MaterialShader::new(mat_program).map_err(|e| format!("{:?}", e))?;
    let line_shader = LineShader::new(line_program).map_err(|e| format!("{:?}", e))?;

    let renderer = Rc::new(WebGlRenderer::new(
        Rc::clone(&context),
        mat_shader,
        line_shader,
    ));
    renderer.configure_context();

    let renderer_clone = Rc::clone(&renderer) as Rc<GameRenderer<Context = WebGlContext>>;
    let make_missile_trail = move || {
        Some(Rc::new(
            MissileTrailRenderer::new(Rc::clone(&renderer_clone), Rgb::new(1.0, 1.0, 1.0)).ok()?,
        ) as Rc<EntityRenderer>)
    };
    let mut state = GameState::new(Box::new(make_missile_trail));

    let raw_gltf = assets
        .get("assets/meshes/ship.glb")
        .map_err(|_| String::from("Unable to retrieve mesh asset"))?;
    let gltf = Gltf::from_reader(Cursor::new(raw_gltf)).map_err(|e| format!("{:?}", e))?;
    let mut loader = GltfLoader::new(Rc::clone(renderer.context()), &gltf);
    let first_mesh = gltf
        .meshes()
        .next()
        .ok_or_else(|| String::from("Unable to find mesh"))?;
    let ship_mesh = loader
        .load_mesh(&first_mesh)
        .map_err(|e| format!("Unable to load mesh: {:?}", e))?;

    let renderer_clone = Rc::clone(&renderer);
    let make_ship_renderer = move |player: &Player| {
        Ok(mapgen::make_ship_mesh_renderer(
            Rc::clone(&renderer_clone) as Rc<GameRenderer<Context = WebGlContext>>,
            &ship_mesh,
            &player.color,
        ))
    };
    let mut mapgen_params = MapgenParams {
        game_state: &mut state,
        width: DEFAULT_MAP_WIDTH,
        height: DEFAULT_MAP_HEIGHT,
        num_players: 2,
        game_renderer: Rc::clone(&renderer) as Rc<GameRenderer<Context = WebGlContext>>,
        make_ship_renderer: Box::new(make_ship_renderer),
    };
    mapgen_params
        .generate_map()
        .map_err(|e| format!("Unable to create map: {:?}", e))?;
    state.next_player();

    let mut game_handle = GameHandle::new(Rc::new(RefCell::new(state)), Rc::clone(&renderer));

    let render_state = Rc::clone(game_handle.game_state());
    let render_frame = move |_milliseconds: f64| {
        renderer
            .render(&mut render_state.borrow_mut())
            .unwrap_or_else(|e| log(&e.to_string()));
    };

    let update_state = Rc::clone(game_handle.game_state());
    let update_input_queue = Rc::clone(game_handle.input_queue());
    let update_interface = Rc::clone(game_handle.interface());
    let update_game = move || {
        let mut queue = update_input_queue.borrow_mut();
        let queue_len = queue.len();
        for event in queue.drain(0..queue_len) {
            if let Err(e) = update_state.borrow_mut().handle_input(&event) {
                log(&e.to_string());
            }
        }
        update_state.borrow_mut().update_missiles();
        if let Some(ref interface) = *update_interface.borrow() {
            interface.update_ui();
        }
    };

    let mut render_callback = Box::new(AnimationFrameCallback::new(render_frame));
    render_callback.start()?;
    game_handle.add_callback(render_callback);
    let mut update_callback = Box::new(IntervalCallback::new(
        update_game,
        (crate::state::TICK_INTERVAL * 1000.0) as i32,
    ));
    update_callback.start()?;
    game_handle.add_callback(update_callback);

    Ok(game_handle)
}
