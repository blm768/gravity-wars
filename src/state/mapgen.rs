use std::f32::consts::PI;
use std::rc::Rc;

use nalgebra::{UnitQuaternion, Vector3};

use crate::meshgen;
use crate::rendering::context::RenderingContext;
use crate::rendering::material::Material;
use crate::rendering::Rgba;
use crate::state::{Entity, EntityRenderer, GameState, Ship};
use crate::state_renderer::{GameRenderer, MeshRenderer};

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
    planet.mass = 100.0;
    planet.renderer = Some(planet_renderer);
    state.entities.push(planet);
    Ok(())
}

pub fn add_ships(state: &mut GameState, ship_renderer: Rc<EntityRenderer>) {
    let mut ship = Entity::new(Vector3::new(0.0, 0.0, 0.0));
    ship.ship = Some(Ship {});
    ship.rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI * 0.5);
    ship.renderer = Some(ship_renderer);
    state.entities.push(ship)
}
