use std::fmt::Debug;
use std::rc::Rc;

use nalgebra::base::dimension::{U2, U3};
use nalgebra::{Isometry, Similarity, Translation, Unit, UnitComplex, UnitQuaternion, Vector3};
use ncollide2d::query::{Proximity, Ray, RayCast};
use ncollide2d::shape::Shape;
use num_complex::Complex;

use crate::rendering::light::SunLight;
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
pub const MISSILE_VELOCITY_SCALE: f32 = 10.0;

/// Gravitational constant
pub const GRAVITATIONAL_CONSTANT: f32 = 5e-10;

pub struct Entity {
    pub transform: EntityTransform,
    pub mass: f32,
    pub collision_shape: Option<Box<dyn Shape<f32>>>,
    pub renderer: Option<Rc<EntityRenderer>>,
    pub missile_trail: Option<MissileTrail>,
    pub ship: Option<Ship>,
}

impl Entity {
    pub fn new(position: Vector3<f32>) -> Entity {
        Entity {
            transform: EntityTransform::at_position(position),
            mass: 0.0,
            collision_shape: None,
            renderer: None,
            missile_trail: None,
            ship: None,
        }
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.transform.position
    }

    /// Returns the gravitational acceleration produced by this entity on a mass at pos
    pub fn gravity_at(&self, pos: &Vector3<f32>) -> Vector3<f32> {
        let difference = self.transform.position - pos;
        let strength = difference.magnitude_squared() * self.mass * GRAVITATIONAL_CONSTANT;
        difference.normalize() * strength
    }

    pub fn collides_with_shape(
        &self,
        other_shape: &dyn Shape<f32>,
        other_transform: &Isometry<f32, U2, UnitComplex<f32>>,
    ) -> bool {
        use ncollide2d::query;
        if let Some(shape) = &self.collision_shape {
            let proximity = query::proximity(
                &self.collision_transform(),
                shape.as_ref(),
                other_transform,
                other_shape,
                std::f32::EPSILON,
            );
            return match proximity {
                Proximity::Disjoint => false,
                _ => true,
            };
        }
        false
    }

    /**
     * Makes a rough mapping from the 3D transform to a 2D transform for collision detection.
     */
    fn collision_transform(&self) -> Isometry<f32, U2, UnitComplex<f32>> {
        let rotated = self.transform.rotation * Vector3::new(1.0, 0.0, 0.0);
        let flat_rotation = UnitComplex::from_complex(Complex::new(rotated.x, rotated.y));
        Isometry::from_parts(Translation::from(self.position().xy()), flat_rotation)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntityTransform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: f32,
}

impl EntityTransform {
    pub fn at_position(position: Vector3<f32>) -> EntityTransform {
        EntityTransform {
            position,
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        }
    }

    pub fn to_similarity(&self) -> Similarity<f32, U3, UnitQuaternion<f32>> {
        Similarity::from_parts(Translation::from(self.position), self.rotation, self.scale)
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

    pub fn time_to_collision(&self, entity: &Entity, solid: bool) -> Option<f32> {
        if let (Some(pos), Some(shape)) = (&self.positions.last(), &entity.collision_shape) {
            let velocity = self.velocity.xy();
            if velocity.magnitude_squared() > 0.0 {
                let ray = Ray::new(pos.xy().into(), velocity.xy());
                let transform = entity.collision_transform();
                shape.toi_with_ray(&transform, &ray, solid)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ship {
    player_id: usize,
}

pub struct Player {
    pub color: Rgb,
}

pub struct WorldLight {
    pub sun: SunLight,
    pub ambient: Rgb,
}

pub type RendererFactory = Box<FnMut() -> Option<Rc<EntityRenderer>>>;

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
            let speed = params.speed * MISSILE_VELOCITY_SCALE;
            let direction = Vector3::new(params.angle.cos(), params.angle.sin(), 0.0);
            let mut trail = MissileTrail {
                time_to_live: MISSILE_TIME_TO_LIVE,
                velocity: speed * direction,
                positions: vec![*ship.position()],
                data_version: 0,
            };
            if let Some(ref shape) = ship.collision_shape {
                let radius = shape.bounding_sphere(&ship.collision_transform()).radius();
                // Make sure we've gotten past the ship's own collision shape. Convex shapes may have multiple intersections before achieving clearance.
                while (trail.positions[0] - ship.position()).magnitude_squared() < radius * radius {
                    if let Some(collision) = trail.time_to_collision(ship, false) {
                        trail.positions[0] +=
                            trail.velocity * collision + trail.velocity.normalize() * 0.01;
                    } else {
                        break;
                    }
                }
            }

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
                Self::update_missile(missile, before.iter().chain(after.iter()));
                if let Some(new_pos) = missile.positions.last() {
                    entity.transform.position = *new_pos;
                }
            }
        }
    }

    fn update_missile<'a, T: Iterator<Item = &'a Entity>>(missile: &mut MissileTrail, others: T) {
        if missile.time_to_live <= 0.0 {
            return;
        }
        if let Some(last_pos) = missile.positions.last() {
            missile.time_to_live -= TICK_INTERVAL;
            for other in others {
                if let Some(toi) = missile.time_to_collision(other, true) {
                    if toi <= TICK_INTERVAL {
                        missile.time_to_live = 0.0;
                        missile.add_position(last_pos + missile.velocity * toi);
                        return;
                    }
                }
                missile.velocity += other.gravity_at(last_pos);
            }
            missile.add_position(last_pos + missile.velocity * TICK_INTERVAL);
        }
    }
}
