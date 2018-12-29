use std::f32::consts::PI;
use std::rc::Rc;

use nalgebra::{UnitQuaternion, Vector3};
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
    pub game_renderer: Rc<GameRenderer<Context = Context>>,
    pub make_ship_renderer: Box<Fn(&Player) -> Result<Rc<EntityRenderer>, ()>>,
}

impl<'a, Context> MapgenParams<'a, Context>
where
    Context: RenderingContext + 'static,
{
    pub fn generate_map(&mut self) -> Result<(), MapgenError> {
        self.add_players();
        self.add_ships()?;
        self.add_planets()?;
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
            metal_factor: 0.0,
            roughness: 1.0,
            extras: None,
        };
        let planet_mesh = meshgen::gen_sphere(1.0, 10, renderer.context(), planet_material)
            .map_err(|_| MapgenError::CouldNotCreatePlanetRenderer)?;
        let planet_renderer = Rc::new(MeshRenderer::new(renderer, planet_mesh));

        let mut planet = Entity::new(Vector3::new(3.0, 0.0, 0.0));
        planet.mass = 100.0;
        planet.renderer = Some(planet_renderer);
        self.game_state.entities.push(planet);
        Ok(())
    }

    fn add_ships(&mut self) -> Result<(), MapgenError> {
        let renderers = self
            .make_player_ship_renderers()
            .map_err(|_| MapgenError::CouldNotCreateShipRenderers)?;
        for (id, _player) in self.game_state.players.iter().enumerate() {
            let mut ship = self.place_entity()?;
            ship.ship = Some(Ship { player_id: id });
            ship.rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI * 0.5);
            ship.renderer = Some(Rc::clone(&renderers[id]));
            self.game_state.entities.push(ship)
        }
        Ok(())
    }

    fn make_player_ship_renderers(&mut self) -> Result<Vec<Rc<EntityRenderer>>, ()> {
        let players = &self.game_state.players;
        let mut renderers = Vec::with_capacity(players.len());
        for player in players.iter() {
            let renderer = (self.make_ship_renderer)(player)?;
            renderers.push(renderer)
        }
        Ok(renderers)
    }

    fn place_entity(&self) -> Result<Entity, MapgenError> {
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        let mut rng = rand::thread_rng();

        for _ in 0..MAX_PLACE_ENTITY_TRIES {
            let x = -half_width + rng.gen::<f32>() * self.width;
            let y = -half_height + rng.gen::<f32>() * self.height;
            let pos = Vector3::new(x, y, 0.0);
            if self
                .game_state
                .iter_entities()
                .map(|e| (e.position - pos).magnitude())
                .all(|d| d > 5.0)
            // TODO: use actual collision radius.
            {
                return Ok(Entity::new(pos));
            }
        }
        Err(MapgenError::CouldNotPlaceEntity)
    }
}

pub fn make_ship_mesh_renderer<Context>(
    renderer: Rc<GameRenderer<Context = Context>>,
    mesh: &Mesh<Context>,
    color: &Rgb,
) -> Rc<EntityRenderer>
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
