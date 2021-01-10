use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use traffloat_client_model::*;

pub mod cube;
pub mod sphere;

pub(crate) fn write() -> TokenStream {
    let mut output = quote!();

    output.extend(quote_mesh(
        &format_ident!("TETRAHEDRON"),
        "Unit tetrahedron",
        sphere::sphere(0),
    ));
    output.extend(quote_mesh(
        &format_ident!("SPHERE{}", 1u32),
        "Unit sphere",
        sphere::sphere(1),
    ));
    output.extend(quote_mesh(
        &format_ident!("SPHERE{}", 2u32),
        "Unit sphere",
        sphere::sphere(2),
    ));
    output.extend(quote_mesh(
        &format_ident!("SPHERE{}", 3u32),
        "Unit sphere",
        sphere::sphere(3),
    ));
    output.extend(quote_mesh(
        &format_ident!("CUBE"),
        "Unit cube",
        cube::cube(),
    ));

    output
}
