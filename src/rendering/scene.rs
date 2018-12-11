use cgmath::{Ortho, Vector3};

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

    pub fn projection(&self) -> Ortho<f32> {
        let scale = self.scale();
        let right = scale * self.aspect_ratio;
        Ortho::<f32> {
            left: self.position.x - right,
            right: self.position.x + right,
            top: self.position.y + scale,
            bottom: self.position.y - scale,
            near: -scale,
            far: scale,
        }
    }
}
