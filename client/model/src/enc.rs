use proc_macro2::TokenStream;
use quote::quote;

use super::*;

#[cfg(target_endian = "little")]
pub fn encode_vertices(slice: &[Vertex]) -> &[u8] {
    let len = slice.len();
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len * size_of::<Vertex>()) }
}

#[cfg(target_endian = "little")]
pub fn encode_faces(slice: &[Face]) -> &[u8] {
    let len = slice.len();
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len * size_of::<Face>()) }
}

pub type Mesh = (Vec<Vertex>, Vec<Face>);

pub fn quote_mesh(name: &proc_macro2::Ident, doc: &str, mesh: Mesh) -> TokenStream {
    let vertices = &mesh.0;
    let faces = &mesh.1;

    let doc = &format!(
        "{}\n\nMesh with {} vertices and {} faces",
        doc,
        vertices.len(),
        faces.len()
    );

    let vertices = proc_macro2::Literal::byte_string(encode_vertices(vertices));
    let faces = proc_macro2::Literal::byte_string(encode_faces(faces));

    quote! {
        #[doc = #doc]
        pub const #name: RawMesh = RawMesh { vertices: #vertices, faces: #faces };
    }
}
