use nalgebra::{Orthographic3, Vector3};

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub aspect_ratio: f32,
    pub log_scale: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vector3::new(0.0, 0.0, 0.0),
            aspect_ratio: 1.0,
            log_scale: 0.0,
        }
    }

    pub fn scale(&self) -> f32 {
        10.0_f32.powf(self.log_scale)
    }

    pub fn projection(&self) -> Orthographic3<f32> {
        let scale = self.scale();
        let right = scale * self.aspect_ratio;
        Orthographic3::new(
            self.position.x - right,
            self.position.x + right,
            self.position.y - scale,
            self.position.y + scale,
            -scale,
            scale,
        )
    }
}
