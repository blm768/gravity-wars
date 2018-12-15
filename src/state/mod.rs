use std::fmt::Debug;
use std::rc::Rc;

use cgmath::Vector3;

use crate::rendering::light::PointLight;
use crate::rendering::scene::Camera;
use crate::rendering::Rgb;
use crate::state::event::{InputEvent, InputEventError};

pub mod event;
pub mod mapgen;

// Maximum time to live (in seconds)
pub const MISSILE_TIME_TO_LIVE: f32 = 30.0;

#[derive(Debug)]
pub struct Entity {
    pub position: Vector3<f32>,
    pub renderer: Option<Rc<EntityRenderer>>,
    pub missile_trail: Option<MissileTrail>,
    pub ship: Option<Ship>,
}

impl Entity {
    pub fn new(position: Vector3<f32>) -> Entity {
        Entity {
            position,
            renderer: None,
            missile_trail: None,
            ship: None,
        }
    }
}

pub trait EntityRenderer: Debug {
    fn render(&self, entity: &Entity, world: &GameState);
}

#[derive(Clone, Debug)]
pub struct MissileTrail {
    pub time_to_live: f32,
    pub velocity: Vector3<f32>,
    pub positions: Vec<Vector3<f32>>,
}

#[derive(Clone, Debug)]
pub struct Ship {}

pub struct GameState {
    pub entities: Vec<Entity>,
    pub camera: Camera,
    pub light: PointLight,
}

impl GameState {
    pub fn new() -> GameState {
        let light = PointLight {
            color: Rgb::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, -3.0),
        };

        GameState {
            entities: Vec::new(),
            camera: Camera::new(),
            light,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn get_ship(&self) -> Option<&Entity> {
        self.entities.iter().find(|e| e.ship.is_some())
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> Result<(), InputEventError> {
        // TODO: clamp movement/scale.
        match event {
            InputEvent::PanCamera(delta) => {
                self.camera.position += Vector3::new(delta.x, delta.y, 0.0);
            }
            InputEvent::ZoomCamera(ref scale) => {
                self.camera.log_scale += scale;
            }
            InputEvent::FireMissile(ref params) => {
                // TODO: validation
                let missile = {
                    let ship = self
                        .get_ship()
                        .ok_or(InputEventError::NoShipToFireMissile)?;
                    let mut entity = Entity::new(ship.position);
                    entity.missile_trail = Some(MissileTrail {
                        time_to_live: MISSILE_TIME_TO_LIVE,
                        velocity: Vector3::new(
                            params.speed * params.angle.cos(),
                            params.speed * params.angle.sin(),
                            0.0,
                        ),
                        positions: Vec::new(),
                    });
                    entity
                };
                self.entities.push(missile);
            }
        }
        Ok(())
    }

    pub fn update_missiles(&mut self) {
        let missiles = self
            .entities
            .iter_mut()
            .filter_map(|e| match e.missile_trail {
                Some(ref mut trail) => Some((&mut e.position, trail)),
                None => None,
            });

        for (pos, missile) in missiles {
            if missile.time_to_live > 0.0 {
                missile.time_to_live -= 1.0; // TODO: figure out time handling properly...
                *pos += missile.velocity;
                missile.positions.push(*pos);
                // TODO: handle collisions.
            }
        }
    }
}
