use std::convert::TryInto;
use std::mem;
use std::rc::Rc;

use crate::rendering::buffer::{Buffer, BufferData, VertexAttributeBinding};
use crate::rendering::context::RenderingContext;
use crate::rendering::material::Material;
use crate::rendering::mesh::{ElementIndices, Mesh, Primitive, PrimitiveGeometry, VertexAttribute};

use nalgebra::Vector3;

pub struct GridMesh {
    positions: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    segments: usize,
}

impl GridMesh {
    pub fn new(segments: usize) -> GridMesh {
        let mut positions = Vec::new();
        let num_verts = segments + 1;
        positions.resize(num_verts * num_verts, Vector3::new(0.0, 0.0, 0.0));
        let normals = positions.clone();
        GridMesh {
            positions,
            normals,
            segments,
        }
    }

    pub fn segments(&self) -> usize {
        self.segments
    }

    pub fn vert_index(&self, row: usize, col: usize) -> Option<usize> {
        if row <= self.segments && col <= self.segments {
            Some(row * (self.segments + 1) + col)
        } else {
            None
        }
    }

    pub fn set_position_at(&mut self, row: usize, col: usize, pos: Vector3<f32>) {
        let index = self.vert_index(row, col).unwrap();
        self.positions[index] = pos;
    }

    pub fn set_normal_at(&mut self, row: usize, col: usize, normal: Vector3<f32>) {
        let index = self.vert_index(row, col).unwrap();
        self.normals[index] = normal;
    }

    /// Returns the index into the vertex array for each corner of each triangle of each face.
    pub fn face_indices<T>(&self) -> Vec<T>
    where
        usize: TryInto<T>,
        <usize as std::convert::TryInto<T>>::Error: std::fmt::Debug,
    {
        let get_index = |row, col| self.vert_index(row, col).unwrap().try_into().unwrap();

        let mut indices: Vec<T> = Vec::with_capacity(self.segments * self.segments * 6);
        for row in 0..self.segments {
            for col in 0..self.segments {
                indices.push(get_index(row, col));
                indices.push(get_index(row, col + 1));
                indices.push(get_index(row + 1, col));
                indices.push(get_index(row + 1, col + 1));
                indices.push(get_index(row + 1, col));
                indices.push(get_index(row, col + 1));
            }
        }
        indices
    }

    pub fn make_primitive<Context: RenderingContext>(
        &self,
        context: &Context,
    ) -> Result<PrimitiveGeometry<Context>, ()> {
        let index_data = self.face_indices::<u16>(); // TODO: use smaller index types where possible.
        let indices = ElementIndices::from_data(&index_data, context)?;

        let attribute_buf = Rc::new(context.make_attribute_buffer()?);
        let mut attribute_data = Vec::<Vector3<f32>>::new();
        attribute_data.reserve(self.positions.len() + self.normals.len());
        for i in 0..self.positions.len() {
            attribute_data.push(self.positions[i]);
            attribute_data.push(self.normals[i]);
        }
        attribute_buf.set_data(attribute_data.as_bytes());

        let vec_size = mem::size_of::<Vector3<f32>>();
        let stride = vec_size * 2;

        let mut pos_binding = VertexAttributeBinding::typed::<Vector3<f32>>(self.positions.len());
        pos_binding.set_stride(stride);
        let positions = VertexAttribute::new(Rc::clone(&attribute_buf), pos_binding);

        let mut normal_binding = VertexAttributeBinding::typed::<Vector3<f32>>(self.normals.len());
        normal_binding.set_offset(vec_size);
        normal_binding.set_stride(stride);
        let normals = VertexAttribute::new(Rc::clone(&attribute_buf), normal_binding);

        PrimitiveGeometry::new(Some(indices), positions, normals)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CubeFace {
    Left,
    Right,
    Bottom,
    Top,
    Back,
    Front,
}

impl CubeFace {
    pub const ALL: &'static [CubeFace] = &[
        CubeFace::Left,
        CubeFace::Right,
        CubeFace::Bottom,
        CubeFace::Top,
        CubeFace::Back,
        CubeFace::Front,
    ];

    pub fn normal(self) -> Vector3<f32> {
        match self {
            CubeFace::Left => Vector3::new(-1.0, 0.0, 0.0),
            CubeFace::Right => Vector3::new(1.0, 0.0, 0.0),
            CubeFace::Bottom => Vector3::new(0.0, -1.0, 0.0),
            CubeFace::Top => Vector3::new(0.0, 1.0, 0.0),
            CubeFace::Back => Vector3::new(0.0, 0.0, -1.0),
            CubeFace::Front => Vector3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn tangent(self) -> Vector3<f32> {
        match self {
            CubeFace::Left => Vector3::new(0.0, 0.0, 1.0),
            CubeFace::Right => Vector3::new(0.0, 0.0, -1.0),
            CubeFace::Bottom => Vector3::new(1.0, 0.0, 0.0),
            CubeFace::Top => Vector3::new(1.0, 0.0, 0.0),
            CubeFace::Back => Vector3::new(-1.0, 0.0, 0.0),
            CubeFace::Front => Vector3::new(1.0, 0.0, 0.0),
        }
    }

    pub fn bitangent(self) -> Vector3<f32> {
        match self {
            CubeFace::Left => Vector3::new(0.0, 1.0, 0.0),
            CubeFace::Right => Vector3::new(0.0, 1.0, 0.0),
            CubeFace::Bottom => Vector3::new(0.0, 0.0, 1.0),
            CubeFace::Top => Vector3::new(0.0, 0.0, -1.0),
            CubeFace::Back => Vector3::new(0.0, 1.0, 0.0),
            CubeFace::Front => Vector3::new(0.0, 1.0, 0.0),
        }
    }
}

/// Maps a location on a cube to the corresponding location on a sphere
pub fn cube_to_sphere(square_loc: &Vector3<f32>) -> Vector3<f32> {
    // Formula from http://mathproofs.blogspot.com/2005/07/mapping-cube-to-sphere.html
    let x_sq = square_loc.x * square_loc.x;
    let y_sq = square_loc.y * square_loc.y;
    let z_sq = square_loc.z * square_loc.z;
    let f = |a: f32, b_sq: f32, c_sq: f32| {
        a * (1.0 - b_sq / 2.0 - c_sq / 2.0 + b_sq * c_sq / 3.0).sqrt()
    };
    Vector3::new(
        f(square_loc.x, y_sq, z_sq),
        f(square_loc.y, z_sq, x_sq),
        f(square_loc.z, x_sq, y_sq),
    )
}

pub fn gen_part_sphere(radius: f32, segments: usize, face: CubeFace) -> GridMesh {
    let min = -radius;
    let step = 2.0 * radius / segments as f32;

    let face_normal = face.normal();
    let face_tangent = face.tangent();
    let face_bitangent = face.bitangent();

    let mut mesh = GridMesh::new(segments);
    let verts_per_dim = segments + 1;
    for row in 0..verts_per_dim {
        let v = min + (row as f32) * step;
        for col in 0..verts_per_dim {
            let u = min + (col as f32) * step;
            let cube_loc = face_normal * radius + u * face_tangent + v * face_bitangent;
            let sphere_loc = cube_to_sphere(&cube_loc);
            mesh.set_position_at(row, col, sphere_loc);
            mesh.set_normal_at(row, col, sphere_loc / radius);
        }
    }
    mesh
}

pub fn gen_sphere<Context>(
    radius: f32,
    segments: usize,
    context: &Context,
    material: Material,
) -> Result<Mesh<Context>, ()>
where
    Context: RenderingContext,
{
    let primitives = CubeFace::ALL
        .iter()
        .map(|f| gen_part_sphere(radius, segments, *f))
        .map(|m| m.make_primitive(context))
        .map(move |g| {
            Ok(Primitive {
                material: material.clone(),
                geometry: Rc::new(g?),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Mesh::new(primitives))
}
