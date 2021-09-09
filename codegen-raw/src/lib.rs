#![feature(drain_filter)]

extern crate proc_macro;

mod system;

#[proc_macro_attribute]
pub fn system(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    system::imp(attr.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
