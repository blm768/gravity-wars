use cgmath::Vector2;

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    PanCamera(Vector2<f32>),
    ZoomCamera(f32),
}
