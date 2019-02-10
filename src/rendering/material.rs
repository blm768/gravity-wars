use std::ops::Deref;
use std::rc::Rc;

use nalgebra::Matrix4;

use crate::rendering;
use crate::rendering::context::RenderingContext;
use crate::rendering::light::{LightShaderInfo, SunLight};
use crate::rendering::shader::{BoundShader, ShaderBindError};
use crate::rendering::shader::{ShaderInfoError, ShaderParamInfo, ShaderProgram};
use crate::rendering::{Rgb, Rgba};

#[derive(Debug)]
pub struct Material<Context: RenderingContext> {
    pub base_color: Rgba,
    pub base_color_texture: Option<Rc<Context::Texture>>,
    pub metal_factor: f32,
    pub roughness: f32,
    pub extras: Option<serde_json::Value>,
}

// TODO: why doesn't #[derive(Clone)] work properly?
impl<Context: RenderingContext> Clone for Material<Context> {
    fn clone(&self) -> Self {
        Material {
            base_color: self.base_color.clone(),
            base_color_texture: self.base_color_texture.clone(),
            metal_factor: self.metal_factor,
            roughness: self.roughness,
            extras: self.extras.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MaterialShaderInfo {
    pub position: ShaderParamInfo,
    pub normal: ShaderParamInfo,

    pub projection: ShaderParamInfo,
    pub model_transform: ShaderParamInfo,
    pub view_transform: ShaderParamInfo,
    pub base_color: Option<ShaderParamInfo>,
    pub metal_factor: Option<ShaderParamInfo>,
    pub roughness: Option<ShaderParamInfo>,

    pub lights: LightShaderInfo,
}

impl MaterialShaderInfo {
    pub fn from_program<Context: RenderingContext>(
        program: &ShaderProgram<RenderingContext = Context>,
    ) -> Result<MaterialShaderInfo, ShaderInfoError> {
        Ok(MaterialShaderInfo {
            position: ShaderParamInfo::attribute(program, "position")?,
            normal: ShaderParamInfo::attribute(program, "normal")?,
            projection: ShaderParamInfo::uniform(program, "projection")?,
            model_transform: ShaderParamInfo::uniform(program, "model")?,
            view_transform: ShaderParamInfo::uniform(program, "view")?,
            base_color: ShaderParamInfo::uniform(program, "material.baseColor").ok(),
            metal_factor: ShaderParamInfo::uniform(program, "material.metal").ok(),
            roughness: ShaderParamInfo::uniform(program, "material.roughness").ok(),
            lights: LightShaderInfo::from_program(program),
        })
    }

    pub fn bind_material<Context: RenderingContext>(
        &self,
        material: &Material<Context>,
        context: &BoundShader<Context>,
    ) {
        if let Some(ref base_color) = self.base_color {
            context.set_uniform_vec4(
                base_color.index,
                rendering::rgba_as_vec4(&material.base_color),
            );
        }
        // TODO: handle textures.
        if let Some(ref metal_factor) = self.metal_factor {
            context.set_uniform_f32(metal_factor.index, material.metal_factor);
        }
        if let Some(ref roughness) = self.roughness {
            context.set_uniform_f32(roughness.index, material.roughness);
        }
    }
}

#[derive(Debug)]
pub struct MaterialShader<Context: RenderingContext> {
    pub program: Rc<Context::ShaderProgram>,
    pub info: MaterialShaderInfo,
}

impl<Context: RenderingContext> MaterialShader<Context> {
    pub fn new(program: Context::ShaderProgram) -> Result<Self, ShaderInfoError> {
        let info = MaterialShaderInfo::from_program(&program)?;
        Ok(MaterialShader {
            program: Rc::new(program),
            info,
        })
    }
}

pub trait MaterialWorldContext {
    fn projection(&self) -> Matrix4<f32>;
    fn view(&self) -> Matrix4<f32>;
    fn sun(&self) -> &SunLight;
    fn ambient(&self) -> Rgb;
}

pub struct BoundMaterialShader<Context: RenderingContext> {
    bound_shader: Context::BoundShader,
    info: MaterialShaderInfo, // TODO: give this a lifetime bound and just borrow here?
}

impl<Context: RenderingContext> BoundMaterialShader<Context> {
    pub fn new(
        context: &Context,
        shader: &MaterialShader<Context>,
        world: &MaterialWorldContext,
    ) -> Result<Self, ShaderBindError> {
        let bound_shader = context.bind_shader(Rc::clone(&shader.program))?;
        let info = &shader.info;
        bound_shader.set_uniform_mat4(info.view_transform.index, world.view());
        bound_shader.set_uniform_mat4(info.projection.index, world.projection());
        info.lights.bind_sun(world.sun(), &bound_shader);
        info.lights.bind_ambient(&world.ambient(), &bound_shader);
        Ok(BoundMaterialShader {
            bound_shader,
            info: shader.info.clone(),
        })
    }

    pub fn info(&self) -> &MaterialShaderInfo {
        &self.info
    }

    pub fn bound_shader(&self) -> &BoundShader<Context> {
        &self.bound_shader
    }
}

impl<Context> Deref for BoundMaterialShader<Context>
where
    Context: RenderingContext,
{
    type Target = BoundShader<Context>;
    fn deref(&self) -> &Self::Target {
        &self.bound_shader
    }
}
