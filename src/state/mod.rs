use std::fmt::Debug;
use std::rc::Rc;

use cgmath::Vector3;

use crate::rendering::light::PointLight;
use crate::rendering::scene::Camera;
use crate::rendering::Rgb;
use crate::state::event::{InputEvent, InputEventError, MissileParams};

pub mod event;
pub mod mapgen;

/// Maximum time to live (in seconds)
pub const MISSILE_TIME_TO_LIVE: f32 = 30.0;
/// Maximum missile velocity (in arbitrary units)
pub const MISSILE_MAX_VELOCITY: f32 = 10.0;
/// Scaling factor from missile velocity units to actual game units per second
pub const MISSILE_VELOCITY_SCALE: f32 = 0.1;
/// Gravitational constant
pub const GRAVITATIONAL_CONSTANT: f32 = 0.0001;

#[derive(Debug)]
pub struct Entity {
    pub position: Vector3<f32>,
    pub mass: f32,
    pub renderer: Option<Rc<EntityRenderer>>,
    pub missile_trail: Option<MissileTrail>,
    pub ship: Option<Ship>,
}

impl Entity {
    pub fn new(position: Vector3<f32>) -> Entity {
        Entity {
            position,
            mass: 0.0,
            renderer: None,
            missile_trail: None,
            ship: None,
        }
    }

    /// Returns the gravitational acceleration produced by this entity on a mass at pos
    pub fn gravity_at(&self, pos: &Vector3<f32>) -> Vector3<f32> {
        use cgmath::InnerSpace;
        let difference = self.position - pos;
        difference.normalize() * (difference.magnitude2() * self.mass * GRAVITATIONAL_CONSTANT)
    }
}

pub trait EntityRenderer: Debug {
    fn render(&self, entity: &Entity, world: &GameState);
}

#[derive(Clone, Debug)]
pub struct MissileTrail {
    pub time_to_live: f32,
    pub velocity: Vector3<f32>,
    positions: Vec<Vector3<f32>>,
    data_version: usize,
}

impl MissileTrail {
    pub fn new(velocity: Vector3<f32>) -> MissileTrail {
        MissileTrail {
            time_to_live: MISSILE_TIME_TO_LIVE,
            velocity,
            positions: Vec::new(),
            data_version: 0,
        }
    }

    pub fn data_version(&self) -> usize {
        self.data_version
    }

    pub fn positions(&self) -> &[Vector3<f32>] {
        &self.positions
    }

    pub fn add_position(&mut self, position: Vector3<f32>) {
        self.data_version += 1;
        self.positions.push(position);
    }
}

#[derive(Clone, Debug)]
pub struct Ship {}

pub type RendererFactory = Box<FnMut() -> Option<Rc<EntityRenderer>>>;

pub struct GameState {
    pub entities: Vec<Entity>,
    pub camera: Camera,
    pub light: PointLight,
    pub make_missile_renderer: RendererFactory,
}

impl GameState {
    pub fn new(make_missile_renderer: RendererFactory) -> GameState {
        let light = PointLight {
            color: Rgb::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, -3.0),
        };

        GameState {
            entities: Vec::new(),
            camera: Camera::new(),
            light,
            make_missile_renderer,
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
                Ok(())
            }
            InputEvent::ZoomCamera(ref scale) => {
                self.camera.log_scale += scale;
                Ok(())
            }
            InputEvent::FireMissile(ref params) => self.fire_missile(params),
        }
    }

    fn fire_missile(&mut self, params: &MissileParams) -> Result<(), InputEventError> {
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
            let mut entity = Entity::new(ship.position);
            let speed = params.speed * MISSILE_VELOCITY_SCALE;
            entity.missile_trail = Some(MissileTrail {
                time_to_live: MISSILE_TIME_TO_LIVE,
                velocity: Vector3::new(speed * params.angle.cos(), speed * params.angle.sin(), 0.0),
                positions: [entity.position].to_vec(),
                data_version: 0,
            });
            entity.position = ship.position;
            entity.renderer = (self.make_missile_renderer)();
            entity
        };
        self.entities.push(missile);
        Ok(())
    }

    pub fn update_missiles(&mut self) {
        let entities = &mut self.entities[..];
        for i in 0..entities.len() {
            // Break off mutable slices to all of the other entities.
            let (before, after) = entities.split_at_mut(i);
            let (entity, after) = after.split_first_mut().unwrap();
            if let Entity {
                position: ref mut pos,
                missile_trail: Some(ref mut missile),
                ..
            } = entity
            {
                if missile.time_to_live > 0.0 {
                    missile.time_to_live -= 1.0; // TODO: figure out time handling properly...
                    *pos += missile.velocity;
                    missile.add_position(*pos);
                    for other in before.iter().chain(after.iter()) {
                        missile.velocity += other.gravity_at(pos);
                    }
                    // TODO: handle collisions.
                }
            }
        }
    }
}
