#![feature(drain_filter)]

extern crate proc_macro;

mod definition;
mod system;

#[proc_macro_attribute]
pub fn system(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    system::imp(attr.into(), input.into()).unwrap_or_else(|err| err.to_compile_error()).into()
}

#[proc_macro_derive(Definition, attributes(hf_serde))]
pub fn definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    definition::imp(input.into()).unwrap_or_else(|err| err.to_compile_error()).into()
}
