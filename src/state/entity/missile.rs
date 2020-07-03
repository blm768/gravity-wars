use nalgebra::Vector3;
use ncollide2d::query::Ray;

use crate::state::constants;
use crate::state::entity::Entity;

#[derive(Clone, Debug)]
pub struct MissileTrail {
    pub player_id: usize,
    pub time_to_live: f32,
    pub velocity: Vector3<f32>,
    positions: Vec<Vector3<f32>>,
    data_version: usize,
}

impl MissileTrail {
    pub fn new(player_id: usize, position: Vector3<f32>, velocity: Vector3<f32>) -> MissileTrail {
        MissileTrail {
            player_id,
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
            entity.ray_time_to_collision(&ray, constants::TICK_INTERVAL, solid)
        } else {
            None
        }
    }

    pub fn update<'a>(
        &mut self,
        other_entities: &mut dyn Iterator<Item = (usize, &'a Entity)>,
    ) -> Option<MissileEvent> {
        if self.time_to_live <= 0.0 {
            return None;
        }

        if let Some(last_pos) = self.positions().last().cloned() {
            self.time_to_live -= constants::TICK_INTERVAL;
            for (i, other) in other_entities {
                if let Some(toi) = self.time_to_collision(other, true) {
                    self.time_to_live = 0.0;
                    self.add_position(last_pos + self.velocity * toi);
                    return Some(MissileEvent::HitEntity(i));
                }
                self.velocity += other.gravity_at(&last_pos);
            }
            self.add_position(last_pos + self.velocity * constants::TICK_INTERVAL);
        }

        if self.time_to_live <= 0.0 {
            Some(MissileEvent::Expired)
        } else {
            None
        }
    }
}

pub enum MissileEvent {
    Expired,
    HitEntity(usize),
}
