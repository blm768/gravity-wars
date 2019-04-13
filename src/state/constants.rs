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
