use std::fmt::Debug;
use std::rc::Rc;

use nalgebra::{Matrix4, Translation, UnitQuaternion, Vector3};

use crate::rendering::light::PointLight;
use crate::rendering::scene::Camera;
use crate::rendering::Rgb;
use crate::state::event::{InputEvent, InputEventError, MissileParams};

pub mod event;
pub mod mapgen;

/// Game state ticks per second
pub const TICKS_PER_SECOND: u32 = 30;
/// Game state tick interval (in seconds)
pub const TICK_INTERVAL: f32 = 1.0 / (TICKS_PER_SECOND as f32);

/// Maximum time to live (in seconds)
pub const MISSILE_TIME_TO_LIVE: f32 = 30.0;
/// Maximum missile velocity (in arbitrary units)
pub const MISSILE_MAX_VELOCITY: f32 = 10.0;
/// Scaling factor from missile velocity units to actual game units per second
pub const MISSILE_VELOCITY_SCALE: f32 = 1.0;

/// Gravitational constant
pub const GRAVITATIONAL_CONSTANT: f32 = 0.0001;

#[derive(Debug)]
pub struct Entity {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub mass: f32,
    pub renderer: Option<Rc<EntityRenderer>>,
    pub missile_trail: Option<MissileTrail>,
    pub ship: Option<Ship>,
}

impl Entity {
    pub fn new(position: Vector3<f32>) -> Entity {
        Entity {
            position,
            rotation: UnitQuaternion::identity(),
            mass: 0.0,
            renderer: None,
            missile_trail: None,
            ship: None,
        }
    }

    pub fn transform(&self) -> Matrix4<f32> {
        (Translation::from(self.position) * self.rotation).to_homogeneous()
    }

    /// Returns the gravitational acceleration produced by this entity on a mass at pos
    pub fn gravity_at(&self, pos: &Vector3<f32>) -> Vector3<f32> {
        let difference = self.position - pos;
        let strength = difference.magnitude_squared() * self.mass * GRAVITATIONAL_CONSTANT;
        difference.normalize() * strength
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
pub struct Ship {
    player_id: usize,
}

pub struct Player {
    pub color: Rgb,
}

pub type RendererFactory = Box<FnMut() -> Option<Rc<EntityRenderer>>>;

pub struct GameState {
    pub entities: Vec<Entity>,
    players: Box<[Player]>,
    current_player: Option<usize>,
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
            players: Box::from([]),
            current_player: None,
            camera: Camera::new(),
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
        self.next_player();
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
                    missile.time_to_live -= TICK_INTERVAL;
                    *pos += missile.velocity * TICK_INTERVAL;
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
