use wasm_bindgen::prelude::*;

use std::io::Cursor;
use std::str;

use cgmath::{Matrix4, Rad, Vector3};
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::{Element, HtmlCanvasElement};
use web_sys::{WebGlRenderingContext, WebGlShader};

use gltf::Gltf;

use glue::asset::{AssetData, AssetLoader, FetchError};
use glue::webgl::buffer::{Buffer, BufferBinding, VertexAttributeBinding};
use glue::webgl::gltf::GltfLoader;
use glue::webgl::{ShaderType, WebGlRenderer};
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
pub fn init_game() -> AssetLoader {
    let assets = AssetLoader::new(start_game);
    assets.load("shaders/vertex.glsl");
    assets.load("shaders/fragment.glsl");
    assets.load("cube.glb");

    assets
}

fn compile_shader_from_asset(
    url: &str,
    asset: Result<&[u8], FetchError>,
    renderer: &WebGlRenderer,
    shader_type: ShaderType,
) -> Result<WebGlShader, String> {
    match asset {
        Ok(ref data) => {
            let text = str::from_utf8(data).unwrap_or("<UTF-8 decoding error>");
            webgl::compile_shader(renderer.context(), shader_type, text)
        }
        Err(error) => Err(format!("Unable to load asset {}: {}", url, error)),
    }
}

#[wasm_bindgen]
pub fn start_game(assets: &AssetData) {
    match try_start_game(assets) {
        Ok(()) => {}
        Err(err) => log(&format!("Error starting game: {}", err)),
    }
}

fn try_start_game(assets: &AssetData) -> Result<(), String> {
    let (canvas_element, canvas) =
        get_canvas().ok_or_else(|| String::from("Unable to find canvas"))?;
    let context = get_webgl_context(&canvas)?;

    let state = GameState::new();

    let renderer = WebGlRenderer::new(canvas_element, canvas, context);
    // TODO: just clear the screen here? Do nothing?
    if let Err(_error) = renderer.render(&state) {
        log("Rendering error");
    }

    let vertex_shader = compile_shader_from_asset(
        "shaders/vertex.glsl",
        assets.get("shaders/vertex.glsl"),
        &renderer,
        ShaderType::Vertex,
    )?;
    let fragment_shader = compile_shader_from_asset(
        "shaders/fragment.glsl",
        assets.get("shaders/fragment.glsl"),
        &renderer,
        ShaderType::Fragment,
    )?;
    let program = webgl::ShaderProgram::link(
        renderer.context().clone(),
        [vertex_shader, fragment_shader].iter(),
    )?;
    let info = MaterialShaderInfo::from_program(&program).map_err(|e| format!("{:?}", e))?;

    let raw_gltf = assets
        .get("cube.glb")
        .map_err(|_| String::from("Unable to retrieve cube"))?;
    let gltf = Gltf::from_reader(Cursor::new(raw_gltf)).map_err(|e| format!("{:?}", e))?;
    let mut loader = GltfLoader::new(renderer.context().clone(), &gltf);
    let first_mesh = loader
        .first_mesh()
        .ok_or_else(|| String::from("Unable to find mesh"))?;
    let mesh = loader
        .load_mesh(&first_mesh)
        .map_err(|_| String::from("Unable to load mesh"))?;
    log(&format!("{:?}", mesh));

    let mut vertices = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    ];

    let buffer = Buffer::new(renderer.context().clone(), BufferBinding::ArrayBuffer)
        .ok_or_else(|| String::from("Unable to create buffer object"))?;
    buffer.set_data(&mut vertices);

    log("Buffers created");

    let position_binding = VertexAttributeBinding::typed::<Vector3<f32>>(vertices.len());
    buffer.bind_to_attribute(info.position.index, &position_binding);

    program.activate();

    renderer.set_viewport();
    renderer.context().clear_color(0.0, 0.0, 0.0, 1.0);
    renderer
        .context()
        .clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    let projection: Matrix4<f32> = state.camera.projection(renderer.aspect_ratio()).into();
    let modelview = Matrix4::<f32>::from_angle_x(Rad(0.5))
        * Matrix4::<f32>::from_angle_z(Rad(0.5))
        * Matrix4::<f32>::from_scale(0.5);

    program.set_uniform_mat4(info.projection.index, projection);
    program.set_uniform_mat4(info.model_view.index, modelview);

    log("Uniforms bound");

    //renderer.context() draw_arrays(WebGlRenderingContext::TRIANGLES, 0, vertices.len() as i32);
    mesh.draw(&program, &info);

    Ok(())
}
