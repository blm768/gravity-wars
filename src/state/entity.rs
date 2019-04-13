use std::fmt::Debug;
use std::rc::Rc;

use nalgebra::base::dimension::{U2, U3};
use nalgebra::{Isometry, Similarity, Translation, UnitComplex, UnitQuaternion, Vector3};
use ncollide2d::query::{Proximity, Ray, RayCast};
use ncollide2d::shape::Shape;
use num_complex::Complex;

use crate::state::{constants, GameState};

pub struct Entity {
    pub transform: EntityTransform,
    pub mass: f32,
    pub collision_shape: Option<Box<dyn Shape<f32>>>,
    pub renderer: Option<Rc<dyn EntityRenderer>>,
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
        let strength =
            difference.magnitude_squared() * self.mass * constants::GRAVITATIONAL_CONSTANT;
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

    pub fn ray_time_to_collision(&self, ray: &Ray<f32>, solid: bool) -> Option<f32> {
        let shape = self.collision_shape.as_ref()?;
        let transform = self.collision_transform();
        shape.toi_with_ray(&transform, &ray, solid)
    }


    /**
     * Makes a rough mapping from the 3D transform to a 2D transform for collision detection.
     */
    pub fn collision_transform(&self) -> Isometry<f32, U2, UnitComplex<f32>> {
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
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>) -> MissileTrail {
        MissileTrail {
            time_to_live: constants::MISSILE_TIME_TO_LIVE,
            velocity,
            positions: vec![position],
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
        let pos = self.positions.last()?;
        let velocity = self.velocity.xy();
        if velocity.magnitude_squared() > 0.0 {
            let ray = Ray::new(pos.xy().into(), velocity.xy());
            entity.ray_time_to_collision(&ray, solid)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub player_id: usize,
}
