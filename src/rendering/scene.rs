use nalgebra::{Isometry, Orthographic3, Translation, UnitQuaternion, Vector3};

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub aspect_ratio: f32,
    pub log_scale: f32,
    pub depth: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::identity(),
            aspect_ratio: 1.0,
            log_scale: 0.0,
            depth: 1.0,
        }
    }

    pub fn scale(&self) -> f32 {
        10.0_f32.powf(self.log_scale)
    }

    pub fn projection(&self) -> Orthographic3<f32> {
        let scale = self.scale();
        let right = scale * self.aspect_ratio;
        let depth = self.depth * 0.5;
        Orthographic3::new(-right, right, -scale, scale, -depth, depth)
    }

    pub fn view(&self) -> Isometry<f32, UnitQuaternion<f32>, 3> {
        Isometry::from_parts(Translation::from(self.position), self.rotation).inverse()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new()
    }
}
