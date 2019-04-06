use nalgebra::{Unit, Vector3};

use crate::rendering;
use crate::rendering::context::RenderingContext;
use crate::rendering::shader::{BoundShader, ShaderParamInfo, ShaderProgram};
use crate::rendering::Rgb;

#[derive(Clone, Debug)]
pub struct LightShaderInfo {
    pub sun: Option<SunLightShaderInfo>,
    pub ambient: Option<ShaderParamInfo>,
}

impl LightShaderInfo {
    pub fn from_program<Context: RenderingContext>(
        program: &dyn ShaderProgram<RenderingContext = Context>,
    ) -> LightShaderInfo {
        LightShaderInfo {
            sun: SunLightShaderInfo::from_program(program),
            ambient: program.uniform("ambient"),
        }
    }

    pub fn bind_sun<Context: RenderingContext>(
        &self,
        sun: &SunLight,
        shader: &dyn BoundShader<Context>,
    ) {
        if let Some(ref sun_info) = self.sun {
            sun_info.bind_light(sun, shader)
        }
    }

    pub fn bind_ambient<Context: RenderingContext>(
        &self,
        ambient: &Rgb,
        shader: &dyn BoundShader<Context>,
    ) {
        if let Some(ref ambient_info) = self.ambient {
            shader.set_uniform_vec3(ambient_info.index, rendering::rgb_as_vec3(ambient));
        }
    }
}

pub struct PointLight {
    pub color: Rgb,
    pub position: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct PointLightShaderInfo {
    pub color: ShaderParamInfo,
    pub position: ShaderParamInfo,
}

impl PointLightShaderInfo {
    pub fn from_program<Context: RenderingContext>(
        program: &dyn ShaderProgram<RenderingContext = Context>,
    ) -> Option<PointLightShaderInfo> {
        Some(PointLightShaderInfo {
            color: program.uniform("light.color")?,
            position: program.uniform("light.position")?,
        })
    }

    pub fn bind_light<Context: RenderingContext>(
        &self,
        light: &PointLight,
        shader: &dyn BoundShader<Context>,
    ) {
        shader.set_uniform_vec3(self.color.index, rendering::rgb_as_vec3(&light.color));
        shader.set_uniform_vec3(self.position.index, light.position);
    }
}

pub struct SunLight {
    pub color: Rgb,
    pub direction: Unit<Vector3<f32>>,
}

#[derive(Clone, Debug)]
pub struct SunLightShaderInfo {
    pub color: ShaderParamInfo,
    pub direction: ShaderParamInfo,
}

impl SunLightShaderInfo {
    pub fn from_program<Context: RenderingContext>(
        program: &dyn ShaderProgram<RenderingContext = Context>,
    ) -> Option<SunLightShaderInfo> {
        Some(SunLightShaderInfo {
            color: program.uniform("sun.color")?,
            direction: program.uniform("sun.direction")?,
        })
    }

    pub fn bind_light<Context: RenderingContext>(
        &self,
        light: &SunLight,
        shader: &dyn BoundShader<Context>,
    ) {
        shader.set_uniform_vec3(self.color.index, rendering::rgb_as_vec3(&light.color));
        shader.set_uniform_vec3(self.direction.index, light.direction.into_inner());
    }
}
