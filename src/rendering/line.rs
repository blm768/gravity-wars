use std::ops::Deref;
use std::rc::Rc;

use nalgebra::{Matrix4, Vector3};

use crate::rendering;
use crate::rendering::buffer::{AttributeBuffer, Buffer, BufferData, VertexAttributeBinding};
use crate::rendering::context::RenderingContext;
use crate::rendering::shader::{BoundShader, ShaderBindError};
use crate::rendering::shader::{ShaderInfoError, ShaderParamInfo};
use crate::rendering::Rgb;

#[derive(Clone, Debug)]
pub struct PolyLine<Context: RenderingContext> {
    buffer: Context::AttributeBuffer,
    binding: VertexAttributeBinding,
    pub color: Rgb,
}

impl<Context: RenderingContext> PolyLine<Context> {
    pub fn new(buffer: Context::AttributeBuffer, color: Rgb) -> PolyLine<Context> {
        PolyLine {
            buffer,
            binding: VertexAttributeBinding::typed::<Vector3<f32>>(0),
            color,
        }
    }

    pub fn set_positions(&mut self, locations: &[Vector3<f32>]) {
        self.buffer.set_data(locations.as_bytes());
        self.binding = VertexAttributeBinding::typed::<Vector3<f32>>(locations.len());
    }

    pub fn draw(&self, context: &BoundLineShader<Context>) {
        context.info.bind_color(&self.color, context.deref());
        self.buffer
            .bind_to_attribute(context.info.position.index, &self.binding);
        context.draw_polyline(self.binding.count);
    }
}

#[derive(Clone, Debug)]
pub struct LineShaderInfo {
    pub position: ShaderParamInfo,
    pub color: ShaderParamInfo,
    pub projection: ShaderParamInfo,
    pub view_transform: ShaderParamInfo,
}

impl LineShaderInfo {
    pub fn from_program<Context>(
        program: &Context::ShaderProgram,
    ) -> Result<LineShaderInfo, ShaderInfoError>
    where
        Context: RenderingContext,
    {
        let position = ShaderParamInfo::attribute(program, "position")?;
        let color = ShaderParamInfo::uniform(program, "color")?;
        let projection = ShaderParamInfo::uniform(program, "projection")?;
        let view_transform = ShaderParamInfo::uniform(program, "view")?;
        Ok(LineShaderInfo {
            position,
            color,
            projection,
            view_transform,
        })
    }

    pub fn bind_color<Context>(&self, color: &Rgb, context: &BoundShader<Context>)
    where
        Context: RenderingContext,
    {
        context.set_uniform_vec3(self.color.index, rendering::rgb_as_vec3(color));
    }
}

#[derive(Debug)]
pub struct LineShader<Context: RenderingContext> {
    pub program: Rc<Context::ShaderProgram>,
    pub info: LineShaderInfo,
}

impl<Context: RenderingContext> LineShader<Context> {
    pub fn new(program: Context::ShaderProgram) -> Result<Self, ShaderInfoError> {
        let info = LineShaderInfo::from_program::<Context>(&program)?;
        Ok(LineShader {
            program: Rc::new(program),
            info,
        })
    }
}

pub trait LineWorldContext {
    fn projection(&self) -> Matrix4<f32>;
    fn view(&self) -> Matrix4<f32>;
}

pub struct BoundLineShader<Context: RenderingContext> {
    bound_shader: Context::BoundShader,
    info: LineShaderInfo, // TODO: give this a lifetime bound and just borrow here?
}

impl<Context: RenderingContext> BoundLineShader<Context> {
    pub fn new(
        context: &Context,
        shader: &LineShader<Context>,
        world: &LineWorldContext,
    ) -> Result<Self, ShaderBindError> {
        let bound_shader = context.bind_shader(Rc::clone(&shader.program))?;
        bound_shader.set_uniform_mat4(shader.info.projection.index, world.projection());
        bound_shader.set_uniform_mat4(shader.info.view_transform.index, world.view());
        Ok(BoundLineShader {
            bound_shader,
            info: shader.info.clone(),
        })
    }

    pub fn info(&self) -> &LineShaderInfo {
        &self.info
    }

    pub fn bound_shader(&self) -> &BoundShader<Context> {
        &self.bound_shader
    }
}

impl<Context> Deref for BoundLineShader<Context>
where
    Context: RenderingContext,
{
    type Target = BoundShader<Context>;
    fn deref(&self) -> &Self::Target {
        &self.bound_shader
    }
}
