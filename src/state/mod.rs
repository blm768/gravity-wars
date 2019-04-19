use std::rc::Rc;

use nalgebra::{Unit, Vector3};
use ncollide2d::query::Ray;

use crate::rendering::light::SunLight;
use crate::rendering::scene::Camera;
use crate::rendering::Rgb;
use crate::state::constants::*;
use crate::state::event::{InputEvent, InputEventError, MissileParams};

pub use crate::state::entity::missile::MissileTrail;
pub use crate::state::entity::*;
pub use crate::state::turn::{Turn, TurnState};

pub mod constants;
pub mod entity;
pub mod event;
pub mod mapgen;
pub mod turn;

pub struct Player {
    pub color: Rgb,
}

pub struct WorldLight {
    pub sun: SunLight,
    pub ambient: Rgb,
}

pub type RendererFactory = Box<dyn FnMut() -> Option<Rc<dyn EntityRenderer>>>;

pub struct GameState {
    pub entities: Vec<Entity>,
    players: Box<[Player]>,
    turn: Option<Turn>,
    pub camera: Camera,
    pub light: WorldLight,
    pub make_missile_renderer: RendererFactory,
}

impl GameState {
    pub fn new(make_missile_renderer: RendererFactory) -> GameState {
        let light = WorldLight {
            sun: SunLight {
                color: Rgb::new(1.0, 1.0, 1.0) * 3.0,
                direction: Unit::new_normalize(Vector3::new(-0.2, -0.1, -1.0)),
            },
            ambient: Rgb::new(1.0, 1.0, 1.0) * 0.3,
        };
        let mut camera = Camera::new();
        camera.depth = mapgen::PLANET_RAD_MAX as f32;

        GameState {
            entities: Vec::new(),
            players: Box::from([]),
            turn: None,
            camera,
            light,
            make_missile_renderer,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn get_ship(&self) -> Option<&Entity> {
        let id = self.turn?.current_player;
        self.entities.iter().find(|ref e| match e.ship {
            Some(ref ship) => ship.player_id == id,
            None => false,
        })
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn set_players(&mut self, players: Box<[Player]>) {
        self.players = players;
        self.turn = None;
    }

    pub fn turn(&self) -> Option<&Turn> {
        self.turn.as_ref()
    }

    pub fn current_player(&self) -> Option<usize> {
        Some(self.turn?.current_player)
    }

    pub fn start_game(&mut self) {
        self.turn = Turn::next_player(
            &None,
            self.players.len(),
            &mut self
                .entities
                .iter()
                .filter_map(|e| Some(e.ship.as_ref()?.player_id)),
        );
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> Result<(), InputEventError> {
        // TODO: clamp movement/scale.
        match event {
            InputEvent::PanCamera(delta) => {
                self.camera.position += Vector3::new(delta.x, delta.y, 0.0);
                Ok(())
            }
            InputEvent::ZoomCamera(scale) => {
                self.camera.log_scale += scale;
                Ok(())
            }
            InputEvent::FireMissile(params) => self.fire_missile(*params),
        }
    }

    fn fire_missile(&mut self, params: MissileParams) -> Result<(), InputEventError> {
        let turn = self.turn.ok_or(InputEventError::CannotFireNow)?;
        if turn.state != TurnState::Aiming {
            return Err(InputEventError::CannotFireNow);
        }
        if !params.angle.is_finite() {
            return Err(InputEventError::InvalidMissileAngle);
        }
        if params.speed < 0.0 || params.speed > MISSILE_MAX_VELOCITY {
            return Err(InputEventError::InvalidMissileSpeed);
        }

        let missile = {
            let ship = self
                .get_ship()
                .ok_or(InputEventError::NoShipToFireMissile)?;
            let speed = params.speed * MISSILE_VELOCITY_SCALE;
            let mut position = *ship.position();
            let direction = Vector3::new(params.angle.cos(), params.angle.sin(), 0.0);
            let velocity = speed * direction;
            if let Some(ref shape) = ship.collision_shape {
                let radius = shape.bounding_sphere(&ship.collision_transform()).radius();
                // Make sure we've gotten past the ship's own collision shape. Convex shapes may have multiple intersections before achieving clearance.
                while (position - ship.position()).magnitude_squared() < radius * radius {
                    let ray = Ray::new(position.xy().into(), velocity.xy());
                    if let Some(collision) = ship.ray_time_to_collision(&ray, false) {
                        position += velocity * collision + direction * 0.01;
                    } else {
                        break;
                    }
                }
            }
            let trail = MissileTrail::new(turn.current_player, position, velocity);

            let mut entity = Entity::new(*ship.position());
            entity.missile_trail = Some(trail);
            entity.renderer = (self.make_missile_renderer)();
            entity
        };
        self.entities.push(missile);
        self.turn.as_mut().unwrap().state = TurnState::Firing;
        Ok(())
    }

    pub fn update_missiles(&mut self) {
        let entities = &mut self.entities[..];
        for i in 0..entities.len() {
            // Break off mutable slices to all of the other entities.
            let (before, after) = entities.split_at_mut(i);
            let (entity, after) = after.split_first_mut().unwrap();
            if let Some(ref mut missile) = entity.missile_trail {
                let others = before.iter().chain(after.iter());
                let event = missile.update(&mut others.clone());
                if let Some(new_pos) = missile.positions().last() {
                    entity.transform.position = *new_pos;
                }
                if let Some(_) = event {
                    self.turn = Turn::next_player(
                        &self.turn,
                        self.players.len(),
                        &mut others.filter_map(|e| Some(e.ship.as_ref()?.player_id)),
                    );
                }
            }
        }
    }
}
