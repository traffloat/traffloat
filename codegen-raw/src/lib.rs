#![feature(drain_filter)]

extern crate proc_macro;

mod gen;
mod system;

#[proc_macro_derive(Gen, attributes(max_size, from_client_only, from_server_only, default))]
pub fn gen_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    gen::imp(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn system(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    system::imp(attr.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
