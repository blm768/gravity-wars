use std::error::Error;
use std::fmt::Display;

use cgmath::Vector2;

#[derive(Clone, Copy, Debug)]
pub enum InputEventError {
    NoShipToFireMissile,
}

impl Display for InputEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Invalid game input event: {}",
            match self {
                InputEventError::NoShipToFireMissile => "No ship that can fire a missile",
            }
        )
    }
}

impl Error for InputEventError {}

#[derive(Clone, Copy, Debug)]
pub struct MissileParams {
    pub angle: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    PanCamera(Vector2<f32>),
    ZoomCamera(f32),
    FireMissile(MissileParams),
}
