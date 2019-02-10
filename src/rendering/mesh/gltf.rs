use std::convert::TryInto;
use std::rc::Rc;

use gltf;
use gltf::accessor::DataType;
use gltf::buffer::Source;
use gltf::{Accessor, Gltf, Semantic};
use image::{ImageError, ImageFormat, RgbImage};

use crate::rendering::buffer::{AttributeType, Buffer, ElementBinding, VertexAttributeBinding};
use crate::rendering::context::RenderingContext;
use crate::rendering::material::Material;
use crate::rendering::mesh::{ElementIndices, Mesh, Primitive, PrimitiveGeometry, VertexAttribute};
use crate::rendering::texture::{SetTextureDataError, Texture};
use crate::rendering::Rgba;

pub struct GltfLoader<'a, Context: RenderingContext> {
    context: Rc<Context>,
    gltf: &'a Gltf, // TODO: make sure all input parameters come from this Gltf instance?
}

#[derive(Clone, Copy, Debug)]
pub enum BufferLoadError {
    InvalidSource,
    UnsupportedStride,
}

#[derive(Debug)]
pub enum ImageLoadError {
    BufferLoadError(BufferLoadError),
    ImageError(ImageError),
    UnsupportedColorFormat,
    UnsupportedImageFormat,
}

#[derive(Debug)]
pub enum MaterialLoadError {
    ImageLoadError(ImageLoadError),
    TextureCreationError,
    SetTextureDataError(SetTextureDataError),
}

#[derive(Debug)]
pub enum PrimitiveLoadError {
    MaterialLoadError(MaterialLoadError),
    MissingAttribute,
}

impl<'a, Context> GltfLoader<'a, Context>
where
    Context: RenderingContext,
{
    pub fn new(context: Rc<Context>, gltf: &'a Gltf) -> GltfLoader<'a, Context> {
        GltfLoader { context, gltf }
    }

    pub fn gltf(&self) -> &'a Gltf {
        self.gltf
    }

    pub fn load_attribute(&mut self, accessor: &Accessor) -> Result<VertexAttribute<Context>, ()> {
        if accessor.sparse().is_some() {
            return Err(()); // TODO: support sparse accessors.
        }

        let buffer = self.context.make_attribute_buffer()?; // TODO: share these between attributes.
        let view = accessor.view();
        let src_buf = view.buffer();

        self.load_buffer(&buffer, &src_buf);

        let binding = VertexAttributeBinding {
            attr_type: AttributeType::Float, // TODO: get from accessor.
            num_components: accessor.dimensions().multiplicity(),
            normalized: accessor.normalized(),
            stride: view.stride().unwrap_or(0),
            offset: view.offset() + accessor.offset(),
            count: accessor.count(),
        };

        Ok(VertexAttribute::new(Rc::new(buffer), binding))
    }

    pub fn load_material(
        &mut self,
        material: &gltf::Material,
    ) -> Result<Material<Context>, MaterialLoadError> {
        let pbr = material.pbr_metallic_roughness();

        let texture_info = pbr.base_color_texture().map(|t| t.texture());
        let image = match texture_info {
            Some(t) => Some(
                self.load_image(&t.source().source())
                    .map_err(|e| MaterialLoadError::ImageLoadError(e))?,
            ),
            None => None,
        };
        let texture = match &image {
            Some(image) => {
                let tex = self
                    .context
                    .make_texture()
                    .map_err(|_| MaterialLoadError::TextureCreationError)?;
                tex.set_image_data(image)
                    .map_err(|e| MaterialLoadError::SetTextureDataError(e))?;
                Some(Rc::new(tex)) // TODO: share images between loads.
            }
            None => None,
        };

        Ok(Material {
            base_color: from4(pbr.base_color_factor()),
            base_color_texture: texture,
            metal_factor: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            extras: material.extras().as_ref().cloned(),
        })
    }

    pub fn load_mesh(&mut self, mesh: &gltf::Mesh) -> Result<Mesh<Context>, PrimitiveLoadError> {
        let primitives: Result<Vec<Primitive<Context>>, PrimitiveLoadError> = mesh
            .primitives()
            .map(|ref p| self.load_primitive(p))
            .collect();
        Ok(Mesh::new(primitives?))
    }

    pub fn load_primitive(
        &mut self,
        primitive: &gltf::Primitive,
    ) -> Result<Primitive<Context>, PrimitiveLoadError> {
        let material = self
            .load_material(&primitive.material())
            .map_err(|e| PrimitiveLoadError::MaterialLoadError(e))?;
        let (_, pos_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Positions)
            .ok_or(PrimitiveLoadError::MissingAttribute)?;
        let (_, normal_accessor) = primitive
            .attributes()
            .find(|(semantic, _)| *semantic == Semantic::Normals)
            .ok_or(PrimitiveLoadError::MissingAttribute)?;
        let indices = match primitive.indices() {
            Some(ref accessor) => Some(
                self.load_indices(accessor)
                    .map_err(|_| PrimitiveLoadError::MissingAttribute)?,
            ),
            None => None,
        };
        let positions = self
            .load_attribute(&pos_accessor)
            .map_err(|_| PrimitiveLoadError::MissingAttribute)?;
        let normals = self
            .load_attribute(&normal_accessor)
            .map_err(|_| PrimitiveLoadError::MissingAttribute)?;
        let geometry = Rc::new(PrimitiveGeometry::new(indices, positions, normals));
        Ok(Primitive { material, geometry })
    }

    fn load_slice_from_view(&self, view: &gltf::buffer::View) -> Result<&[u8], BufferLoadError> {
        if view.stride().unwrap_or(0) != 0 {
            return Err(BufferLoadError::UnsupportedStride);
        }
        let blob = self.gltf.blob.as_ref();
        let data = match view.buffer().source() {
            Source::Bin => blob.map(|vec| &vec[..]).unwrap_or(&[]),
            Source::Uri(_) => Err(BufferLoadError::InvalidSource)?,
        };
        Ok(&data[view.offset()..view.offset() + view.length()])
    }

    fn load_buffer(&self, gl_buf: &Buffer<RenderingContext = Context>, src_buf: &gltf::Buffer) {
        let blob = self.gltf.blob.as_ref();
        let data = match src_buf.source() {
            Source::Bin => blob.map(|vec| &vec[..]).unwrap_or(&[]),
            Source::Uri(_) => &[], // TODO: implement.
        };
        gl_buf.set_data(&data[0..src_buf.length()]);
    }

    fn load_indices(&mut self, accessor: &Accessor) -> Result<ElementIndices<Context>, ()> {
        let buffer = self.context.make_index_buffer()?;
        let view = accessor.view();
        self.load_buffer(&buffer, &view.buffer());
        let attr_type: AttributeType = accessor.data_type().into();
        Ok(ElementIndices::new(
            Rc::new(buffer),
            ElementBinding {
                count: accessor.count(),
                index_type: attr_type.try_into()?,
                offset: view.offset() + accessor.offset(),
            },
        ))
    }

    fn load_image(&mut self, source: &gltf::image::Source) -> Result<RgbImage, ImageLoadError> {
        use gltf::image::Source;
        match source {
            Source::View { view, mime_type } => {
                let buf = self
                    .load_slice_from_view(view)
                    .map_err(|e| ImageLoadError::BufferLoadError(e))?;
                let image_format = image_format_from_mime(mime_type)
                    .ok_or(ImageLoadError::UnsupportedImageFormat)?;
                let image = image::load_from_memory_with_format(buf, image_format)
                    .map_err(|e| ImageLoadError::ImageError(e))?;
                Ok(image.to_rgb())
            }
            Source::Uri { .. } => Err(ImageLoadError::BufferLoadError(
                BufferLoadError::InvalidSource,
            )),
        }
    }
}

impl From<DataType> for AttributeType {
    fn from(data_type: gltf::accessor::DataType) -> Self {
        match data_type {
            DataType::I8 => AttributeType::Byte,
            DataType::U8 => AttributeType::UnsignedByte,
            DataType::I16 => AttributeType::Short,
            DataType::U16 => AttributeType::UnsignedShort,
            DataType::U32 => AttributeType::UnsignedInt,
            DataType::F32 => AttributeType::Float,
        }
    }
}

fn from4(c: [f32; 4]) -> Rgba {
    Rgba::new(c[0], c[1], c[2], c[3])
}

fn image_format_from_mime(mime_type: &str) -> Option<ImageFormat> {
    // TODO: support proper MIME type handling?
    match mime_type {
        "image/png" => Some(ImageFormat::PNG),
        "image/jpeg" => Some(ImageFormat::JPEG),
        "image/gif" => Some(ImageFormat::GIF),
        "image/webp" => Some(ImageFormat::WEBP),
        "image/tiff" => Some(ImageFormat::TIFF),
        "image/tga" | "image/x-tga" => Some(ImageFormat::TGA),
        "image/bmp" => Some(ImageFormat::BMP),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some(ImageFormat::ICO),
        _ => None,
    }
}
