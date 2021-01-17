use super::*;

pub fn cube() -> Mesh {
    let endpoints = [-1_f32, 1_f32];
    let mut vertices = vec![Vertex([0., 0., 0.]); 8];
    let mut normals = vec![Normal([0., 0., 0.]); 8];
    let mut colors = vec![Color([0., 0., 0.]); 8];

    for (xi, &x) in endpoints.iter().enumerate() {
        for (yi, &y) in endpoints.iter().enumerate() {
            for (zi, &z) in endpoints.iter().enumerate() {
                let index = (xi << 2) | (yi << 1) | zi;
                vertices[index] = Vertex([x, y, z]);
                let ratio = 1. / 3_f32.sqrt();
                normals[index] = Normal([x * ratio, y * ratio, z * ratio]);
                colors[index] = Color([0.5; 3]);
            }
        }
    }

    let mut faces = Vec::new();

    fn quad(faces: &mut Vec<Face>, a: FaceIndex, b: FaceIndex, c: FaceIndex, d: FaceIndex) {
        faces.push(Face([a, b, c]));
        faces.push(Face([c, d, a]));
    }

    quad(&mut faces, 0b000, 0b001, 0b011, 0b010);
    quad(&mut faces, 0b100, 0b101, 0b111, 0b110);
    quad(&mut faces, 0b000, 0b001, 0b101, 0b100);
    quad(&mut faces, 0b010, 0b011, 0b111, 0b110);
    quad(&mut faces, 0b000, 0b100, 0b110, 0b010);
    quad(&mut faces, 0b001, 0b101, 0b111, 0b011);

    (vertices, normals, faces, colors)
}
