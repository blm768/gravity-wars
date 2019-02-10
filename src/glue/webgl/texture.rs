use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::slice;

use image::RgbImage;
use web_sys::{WebGlRenderingContext, WebGlTexture};

use crate::glue::webgl::WebGlContext;
use crate::rendering::texture;
use crate::rendering::texture::SetTextureDataError;

pub struct Texture {
    context: Rc<WebGlRenderingContext>,
    texture: WebGlTexture,
}

impl Texture {
    pub fn new(context: Rc<WebGlRenderingContext>) -> Option<Texture> {
        let texture = context.create_texture()?;
        Some(Texture { context, texture })
    }

    fn bind(&self) {
        self.context
            .bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
    }
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Texture")
    }
}

impl texture::Texture for Texture {
    type RenderingContext = WebGlContext;

    fn set_image_data(&self, image: &RgbImage) -> Result<(), SetTextureDataError> {
        self.bind();
        let data: &[u8] = &*image;
        let data_mut = unsafe { slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len()) };
        self.context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGlRenderingContext::TEXTURE_2D,
                0,
                WebGlRenderingContext::RGB as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                WebGlRenderingContext::RGB,
                WebGlRenderingContext::UNSIGNED_BYTE,
                Some(data_mut),
            )
            .map_err(|_| SetTextureDataError)
    }
}
