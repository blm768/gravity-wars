use std::f32::consts::PI;
use std::rc::Rc;

use nalgebra::{Isometry, Point2, Translation, UnitComplex, UnitQuaternion, Vector2, Vector3};
use ncollide2d::shape::{Ball, Polyline, Shape};
use rand::distributions::{Distribution, Normal};
use rand::{self, Rng};

use crate::meshgen;
use crate::rendering::context::RenderingContext;
use crate::rendering::material::Material;
use crate::rendering::mesh::Mesh;
use crate::rendering::{Rgb, Rgba};
use crate::state::{Entity, EntityRenderer, GameState, Player, Ship};
use crate::state_renderer::{GameRenderer, MeshRenderer};

/// Default player colors
pub const PLAYER_COLORS: &[(f32, f32, f32)] = &[(1.0, 0.0, 0.0), (0.0, 0.0, 1.0), (1.0, 1.0, 0.0)];
// Maximum number of tries to place an entity
const MAX_PLACE_ENTITY_TRIES: usize = 256;

// Mean planet radius
const PLANET_RAD_MEAN: f64 = 12.0;
// Standard deviation of planet radius
const PLANET_RAD_STD_DEV: f64 = 5.0;
// Minimum planet radius
const PLANET_RAD_MIN: f64 = 0.1;
// Mean planet area
const PLANET_AREA_MEAN: f64 = std::f64::consts::PI * PLANET_RAD_MEAN * PLANET_RAD_MEAN;
// Mean number of planets per square unit of map space
const PLANET_FREQ_MEAN: f64 = 0.10 / PLANET_AREA_MEAN;
// Standard deviation of planets per square unit of map space
const PLANET_FREQ_STD_DEV: f64 = 0.05 / PLANET_AREA_MEAN;
// Mean planet density
const PLANET_DENS_MEAN: f64 = 3.0;
// Standard deviation of planet density
const PLANET_DENS_STD_DEV: f64 = 1.0;

#[derive(Clone, Copy, Debug)]
pub enum MapgenError {
    CouldNotPlaceEntity,
    CouldNotCreatePlanetRenderer,
    CouldNotCreateShipRenderers,
}

pub struct MapgenParams<'a, Context>
where
    Context: RenderingContext + 'static,
{
    pub game_state: &'a mut GameState,
    pub width: f32,
    pub height: f32,
    pub num_players: usize,
    pub game_renderer: Rc<dyn GameRenderer<Context = Context>>,
    pub ship_mesh: &'a Mesh<Context>,
    pub make_ship_renderer: Box<dyn Fn(&Player) -> Result<Rc<dyn EntityRenderer>, ()>>,
}

impl<'a, Context> MapgenParams<'a, Context>
where
    Context: RenderingContext + 'static,
{
    pub fn generate_map(&mut self) -> Result<(), MapgenError> {
        self.add_players();
        self.add_planets()?;
        self.add_ships()?;
        Ok(())
    }

    fn add_players(&mut self) {
        let mut players = Vec::with_capacity(self.num_players);
        for i in 0..self.num_players {
            let player = Player {
                color: PLAYER_COLORS[i % PLAYER_COLORS.len()].into(),
            };
            players.push(player);
        }
        self.game_state.set_players(players.into());
    }

    fn add_planets(&mut self) -> Result<(), MapgenError> {
        let renderer = Rc::clone(&self.game_renderer);
        // TODO: probably don't build the planet renderer here.
        // TODO: break down primitives so we can more easily share buffers between planets that have different materials.
        let planet_material = Material {
            base_color: Rgba::new(0.0, 0.0, 1.0, 1.0),
            base_color_texture: None,
            metal_factor: 0.0,
            roughness: 1.0,
            extras: None,
        };
        let planet_mesh = meshgen::gen_sphere(1.0, 10, renderer.context(), planet_material)
            .map_err(|_| MapgenError::CouldNotCreatePlanetRenderer)?;
        let planet_renderer = Rc::new(MeshRenderer::new(renderer, planet_mesh));

        let num_planets = {
            let distribution = Normal::new(PLANET_FREQ_MEAN, PLANET_FREQ_STD_DEV);
            let density = distribution.sample(&mut rand::thread_rng()) as f32;
            let count = (density * self.width * self.height).round() as usize;
            count.max(1)
        };

        let radius_distribution = Normal::new(PLANET_RAD_MEAN, PLANET_RAD_STD_DEV);
        let density_distribution = Normal::new(PLANET_DENS_MEAN, PLANET_DENS_STD_DEV);
        for _ in 0..num_planets {
            let radius = radius_distribution
                .sample(&mut rand::thread_rng())
                .max(PLANET_RAD_MIN) as f32;
            let shape = Box::new(Ball::new(radius));
            if let Ok(mut planet) = self.place_entity(shape) {
                let density = density_distribution
                    .sample(&mut rand::thread_rng())
                    .max(0.0) as f32;
                let volume = (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3);
                planet.mass = volume * density;
                planet.transform.scale = radius;
                planet.renderer = Some(Rc::clone(&planet_renderer) as Rc<dyn EntityRenderer>);
                self.game_state.entities.push(planet);
            } else {
                crate::glue::log(&format!("Unable to place planet with radius {}", radius));
            }
        }

        Ok(())
    }

    fn add_ships(&mut self) -> Result<(), MapgenError> {
        let renderers = self
            .make_player_ship_renderers()
            .map_err(|_| MapgenError::CouldNotCreateShipRenderers)?;
        for (id, _player) in self.game_state.players.iter().enumerate() {
            let mesh_shape = make_collision_shape(self.ship_mesh);
            let shape = mesh_shape.unwrap_or_else(|| Box::new(Ball::new(0.5)));
            let mut ship = self.place_entity(shape)?;
            ship.ship = Some(Ship { player_id: id });
            ship.transform.rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI * 0.5);
            ship.renderer = Some(Rc::clone(&renderers[id]));
            self.game_state.entities.push(ship)
        }
        Ok(())
    }

    fn make_player_ship_renderers(&mut self) -> Result<Vec<Rc<dyn EntityRenderer>>, ()> {
        let players = &self.game_state.players;
        let mut renderers = Vec::with_capacity(players.len());
        for player in players.iter() {
            let renderer = (self.make_ship_renderer)(player)?;
            renderers.push(renderer)
        }
        Ok(renderers)
    }

    fn place_entity(&self, shape: Box<dyn Shape<f32>>) -> Result<Entity, MapgenError> {
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        let mut rng = rand::thread_rng();

        for _ in 0..MAX_PLACE_ENTITY_TRIES {
            let x = rng.gen_range(-half_width, half_width);
            let y = rng.gen_range(-half_height, half_height);
            let pos = Vector2::new(x, y);
            let transform = Isometry::from_parts(Translation::from(pos), UnitComplex::identity());
            if self
                .game_state
                .iter_entities()
                .all(|e| !e.collides_with_shape(shape.as_ref(), &transform))
            {
                let mut entity = Entity::new(Vector3::new(pos.x, pos.y, 0.0));
                entity.collision_shape = Some(shape);
                return Ok(entity);
            }
        }
        Err(MapgenError::CouldNotPlaceEntity)
    }
}

pub fn make_ship_mesh_renderer<Context>(
    renderer: Rc<dyn GameRenderer<Context = Context>>,
    mesh: &Mesh<Context>,
    color: &Rgb,
) -> Rc<dyn EntityRenderer>
where
    Context: RenderingContext + 'static,
{
    let mut new_mesh: Mesh<Context> = mesh.clone();
    for mut primitive in new_mesh.primitives.iter_mut() {
        if let Some(ref extra) = primitive.material.extras {
            if let Some(team_color) = extra.get("team_color") {
                if let Some(1) = team_color.as_u64() {
                    primitive.material.base_color = color.alpha(1.0);
                }
            }
        }
    }
    Rc::new(MeshRenderer::new(renderer, new_mesh))
}

fn make_collision_shape<Context: RenderingContext>(
    mesh: &Mesh<Context>,
) -> Option<Box<dyn Shape<f32>>> {
    let extras = mesh.extras.as_ref()?;
    let collision = extras.get("collision_shape")?.as_array()?;
    let mut points: Vec<Point2<f32>> = Vec::new();
    points.reserve(collision.len() / 2);
    let mut iter = collision.iter();
    while let Some(next_val) = iter.next() {
        let x = next_val.as_f64()? as f32;
        let y = iter.next()?.as_f64()? as f32;
        points.push(Point2::new(x, y));
    }
    Some(Box::new(Polyline::new(points, None)))
}
