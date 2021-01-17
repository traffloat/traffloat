//! Model sharing

use std::convert::TryInto;

/// A GL-renderable model
#[derive(Debug, codegen::Gen)]
pub struct Model {
    /// The vertices used in the model
    pub vertices: Vec<Vertex>,
    /// The vertex indices for triangles in the model
    pub faces: Vec<Triangle>,
}

impl Model {
    /// Conversion to buffers.
    ///
    /// # Errors
    /// If some parameters are out of bounds,
    /// a human-readable string representing the error is returned.
    pub fn to_buffers(&self) -> Result<Buffers, &'static str> {
        let mut positions = Vec::with_capacity(self.vertices.len() * 3);
        let mut normals = Vec::with_capacity(self.vertices.len() * 3);
        let mut colors = Vec::with_capacity(self.vertices.len() * 3);
        let mut shininesses = Vec::with_capacity(self.vertices.len());
        let mut reflectances = Vec::with_capacity(self.vertices.len() * 3);

        for vertex in &self.vertices {
            for &f in &vertex.position {
                if !(-1.0..=1.0).contains(&f) {
                    return Err("Position must be within [-1, 1]");
                }
                positions.push(f);
            }
            let norm = vertex.normal.iter().map(|x| x * x).sum::<f32>();
            for &f in &vertex.normal {
                normals.push(f / norm);
            }
            for &f in &vertex.color {
                if !(0.0..=1.0).contains(&f) {
                    return Err("Color must be within [0, 1]");
                }
                colors.push(f);
            }
            shininesses.push(vertex.shininess);
            reflectances.extend(vertex.reflectance.iter().copied());
        }

        let mut faces = Vec::new();
        for &face in &self.faces {
            let [i, j, k] = face.vertices;
            if i == j || j == k || i == k {
                return Err("Triangle must have distinct vertices");
            }
            for &index in &[i, j, k] {
                if !(0..self.vertices.len()).contains(&index.try_into().expect("u16 <= usize")) {
                    return Err("Triangle contains invalid index");
                }
                faces.push(index);
            }
        }
        Ok(Buffers {
            positions,
            normals,
            colors,
            shininesses,
            reflectances,
            faces,
        })
    }
}

/// A vertex in a model.
#[derive(Debug, codegen::Gen)]
pub struct Vertex {
    /// The position of the vertex. Must be within `[-1, 1]^3`.
    pub position: [f32; 3],
    /// The normal vector of the vertex. Must be a unit vector (will be normalized).
    pub normal: [f32; 3],
    /// The base RGB of the vertex. Must be within `[0, 1]^3`.
    pub color: [f32; 3],
    /// Phong's specular shininess of the vertex. Must be
    pub shininess: f32,
    /// Phong's diffuse, specular and ambient reflectances.
    pub reflectance: [f32; 3],
}

/// A triangle to render
#[derive(Debug, Clone, Copy, codegen::Gen)]
pub struct Triangle {
    /// The index of vertices to render
    pub vertices: [u16; 3],
}

/// The buffers to render
#[derive(Debug)]
pub struct Buffers {
    /// The position buffer
    pub positions: Vec<f32>,
    /// The normals buffer
    pub normals: Vec<f32>,
    /// The colors buffer
    pub colors: Vec<f32>,
    /// The shininess buffer
    pub shininesses: Vec<f32>,
    /// The reflectance buffer
    pub reflectances: Vec<f32>,
    /// The index buffer
    pub faces: Vec<u16>,
}
