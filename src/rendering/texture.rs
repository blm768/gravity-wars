use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use image::RgbImage;

use crate::rendering::context::RenderingContext;

#[derive(Clone, Copy, Debug)]
pub struct SetTextureDataError;

impl Display for SetTextureDataError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unable to set texture data")
    }
}

impl Error for SetTextureDataError {}

pub trait Texture: Debug {
    type RenderingContext: RenderingContext + ?Sized;

    fn set_image_data(&self, image: &RgbImage) -> Result<(), SetTextureDataError>;
}
