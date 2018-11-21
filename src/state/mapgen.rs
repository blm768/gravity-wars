use std::rc::Rc;

use cgmath::Vector3;

use meshgen;
use rendering::context::RenderingContext;
use rendering::material::Material;
use rendering::Rgba;
use state::{Entity, EntityRenderer, GameState};
use state_renderer::{GameRenderer, MeshRenderer};

pub fn generate_map(_state: &mut GameState) {}

pub fn add_planets<Context>(
    state: &mut GameState,
    renderer: Rc<GameRenderer<Context = Context>>,
) -> Result<(), ()>
where
    Context: RenderingContext + 'static,
{
    // TODO: probably don't build the renderer here.
    // TODO: break down primitives so we can more easily share buffers between planets that have different materials.
    let planet_material = Material {
        base_color: Rgba::new(0.0, 0.0, 1.0, 1.0),
        metal_factor: 0.0,
        roughness: 1.0,
    };
    let planet_mesh = meshgen::gen_sphere(1.0, 10, renderer.context(), planet_material)?;
    let planet_renderer = Rc::new(MeshRenderer::new(renderer, planet_mesh));

    let mut planet = Entity::new(Vector3::new(3.0, 0.0, 0.0));
    planet.renderer = Some(planet_renderer);
    state.entities.push(planet);
    Ok(())
}

pub fn add_ships(state: &mut GameState, ship_renderer: Rc<EntityRenderer>) {
    let mut ship = Entity::new(Vector3::new(0.0, 0.0, 0.0));
    ship.renderer = Some(ship_renderer);
    state.entities.push(ship)
}
