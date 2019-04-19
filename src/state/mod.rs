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

pub mod constants;
pub mod entity;
pub mod event;
pub mod mapgen;

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
    current_player: Option<usize>,
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
            current_player: None,
            camera,
            light,
            make_missile_renderer,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn get_ship(&self) -> Option<&Entity> {
        match self.current_player {
            Some(id) => self.entities.iter().find(|ref e| match e.ship {
                Some(ref ship) => ship.player_id == id,
                None => false,
            }),
            None => None,
        }
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn set_players(&mut self, players: Box<[Player]>) {
        self.players = players;
        self.current_player = None;
    }

    pub fn current_player(&self) -> Option<usize> {
        self.current_player
    }

    pub fn next_player(&mut self) {
        if self.players.len() == 0 {
            self.current_player = None;
            return;
        }

        let next_player = match self.current_player {
            Some(id) => (id + 1) % self.players.len(),
            None => 0,
        };

        let skip_to_next = self
            .entities
            .iter()
            .filter_map(|e| e.ship.as_ref())
            .filter(|s| s.player_id < self.players.len())
            .map(|s| (s.player_id + self.players.len() - next_player) % self.players.len())
            .min();
        self.current_player = skip_to_next.map(|s| next_player + s);
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
            let trail = MissileTrail::new(position, velocity);

            let mut entity = Entity::new(*ship.position());
            entity.missile_trail = Some(trail);
            entity.renderer = (self.make_missile_renderer)();
            entity
        };
        self.entities.push(missile);
        self.next_player();
        Ok(())
    }

    pub fn update_missiles(&mut self) {
        let entities = &mut self.entities[..];
        for i in 0..entities.len() {
            // Break off mutable slices to all of the other entities.
            let (before, after) = entities.split_at_mut(i);
            let (entity, after) = after.split_first_mut().unwrap();
            if let Some(ref mut missile) = entity.missile_trail {
                missile.update(&mut before.iter().chain(after.iter()));
                if let Some(new_pos) = missile.positions().last() {
                    entity.transform.position = *new_pos;
                }
            }
        }
    }
}
