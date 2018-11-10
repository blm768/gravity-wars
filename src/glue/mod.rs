use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use std::str;

use cgmath::{Matrix4, Rad, Vector3};
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::{Element, HtmlCanvasElement};
use web_sys::{WebGlRenderingContext, WebGlShader};

use gltf::Gltf;

use glue::asset::{AssetData, AssetLoader, FetchError};
use glue::webgl::game_renderer::WebGlRenderer;
use glue::webgl::{ShaderType, WebGlContext};
use rendering::light::PointLight;
use rendering::material::MaterialShader;
use rendering::mesh::gltf::GltfLoader;
use rendering::Rgb;
use state::mapgen;
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
    assets.load("assets/meshes/ship.glb");

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
            webgl::compile_shader(renderer.gl_context(), shader_type, text)
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
    let gl_context = get_webgl_context(&canvas)?;
    let context = WebGlContext::new(canvas_element, canvas, gl_context);

    let mut state = GameState::new();
    mapgen::generate_map(&mut state);

    let renderer = WebGlRenderer::new(context);
    renderer.configure_context();
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
        renderer.gl_context().clone(),
        [vertex_shader, fragment_shader].iter(),
    )?;
    let mat_shader = MaterialShader::new(Box::new(program)).map_err(|e| format!("{:?}", e))?;

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
    //log(&format!("{:?}", mesh));

    let light = PointLight {
        color: Rgb::new(1.0, 1.0, 1.0),
        position: Vector3::new(0.0, 0.0, -3.0),
    };

    let window = web_sys::window().ok_or_else(|| String::from("No window object"))?;

    let render_frame = move |milliseconds: f64| {
        mat_shader.program.activate();

        renderer.context().set_viewport();
        renderer.gl_context().clear_color(0.5, 0.5, 0.5, 1.0);
        renderer.gl_context().clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        let projection: Matrix4<f32> = state
            .camera()
            .projection(renderer.context().aspect_ratio())
            .into();
        let modelview = Matrix4::<f32>::from_angle_y(Rad((milliseconds / 1000.0) as f32))
            * Matrix4::<f32>::from_angle_x(Rad(0.5))
            * Matrix4::<f32>::from_angle_z(Rad(0.5));

        mat_shader
            .program
            .set_uniform_mat4(mat_shader.info.projection.index, projection);
        mat_shader
            .program
            .set_uniform_mat4(mat_shader.info.model_view.index, modelview);

        mat_shader.bind_light(&light);

        mesh.draw(renderer.context(), &mat_shader);
    };

    // TODO: encapsulate this mess.
    let render_loop: Rc<RefCell<Option<Box<Fn(f64)>>>> = Rc::new(RefCell::new(None));
    let render_loop_clone = render_loop.clone();
    let render_loop_cloned_closure: Closure<Fn(f64)> = Closure::new(move |milliseconds: f64| {
        if let Some(func) = render_loop_clone.borrow().as_ref() {
            func(milliseconds);
        }
    });
    {
        let mut render_loop_mut = render_loop.borrow_mut();
        *render_loop_mut = Some(Box::new(move |milliseconds: f64| {
            render_frame(milliseconds);
            let result = web_sys::window()
                .unwrap()
                .request_animation_frame(render_loop_cloned_closure.as_ref().unchecked_ref());
            if result.is_err() {
                log("Error in window.requestAnimationFrame()");
            }
        }));
    };

    let closure: Closure<Fn(f64)> = Closure::new(move |milliseconds: f64| {
        if let Some(func) = render_loop.borrow().as_ref() {
            func(milliseconds);
        }
    });

    let _handle = window
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .map_err(|_| String::from("Error in window.requestAnimationFrame()"))?;

    closure.forget(); // TODO: find a cleaner way to do this.

    Ok(())
}
