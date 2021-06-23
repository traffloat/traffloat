#![feature(drain_filter)]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Result;

#[proc_macro_derive(Gen, attributes(max_size, from_client_only, from_server_only))]
pub fn gen_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    gen_imp(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn gen_imp(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<syn::DeriveInput>(input)?;

    let has_attr = |str: &str| input.attrs.iter().any(|attr| attr.path.is_ident(str));

    let (client_source_cfg, client_sink_cfg, server_source_cfg, server_sink_cfg) =
        match (has_attr("from_client_only"), has_attr("from_server_only")) {
            (true, false) => (
                quote!(#[cfg(feature = "client")]),
                quote!(),
                quote!(),
                quote!(#[cfg(feature = "server")]),
            ),
            (false, true) => (
                quote!(),
                quote!(#[cfg(feature = "client")]),
                quote!(#[cfg(feature = "server")]),
                quote!(),
            ),
            _ => (quote!(), quote!(), quote!(), quote!()),
        };

    let name = &input.ident;
    let ident_hash = hash(name);

    let generics = &input.generics;
    let generic_names = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(param) => Ok(&param.ident),
            param => Err(syn::Error::new_spanned(param, "Unsupported generic type")),
        })
        .collect::<Result<Vec<_>>>()?;
    let generic_names = if generics.params.is_empty() {
        quote!()
    } else {
        quote!(<#(#generic_names),*>)
    };

    Ok(match &input.data {
        syn::Data::Struct(data) => {
            let mut write_fields = Vec::with_capacity(data.fields.len());
            let mut read_fields = Vec::with_capacity(data.fields.len());
            let mut cksum_fields = Vec::with_capacity(data.fields.len());

            for (i, field) in data.fields.iter().enumerate() {
                let i_lit = proc_macro2::Literal::usize_unsuffixed(i);

                let field_name = match field.ident.as_ref() {
                    Some(ident) => ident.to_token_stream(),
                    None => {
                        quote!(#i_lit)
                    }
                };
                let field_key = match field.ident.as_ref() {
                    Some(ident) => quote!(#ident: ),
                    None => quote!(),
                };
                let field_name_hash = match field.ident.as_ref() {
                    Some(ident) => hash(ident),
                    None => i as u128,
                };
                let field_ty = &field.ty;

                write_fields.push(quote! {
                    crate::proto::BinWrite::write(&self.#field_name, &mut *buf);
                });
                read_fields.push(quote! {
                    #field_key crate::proto::BinRead::read(&mut *buf)?
                });
                cksum_fields.push(quote! {
                    output = output.wrapping_add(#field_name_hash);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);

                    output = output.wrapping_add(<#field_ty as crate::proto::ProtoType>::CHECKSUM);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                });
            }

            let read_fields = match &data.fields {
                syn::Fields::Named(_) => quote!({#(#read_fields),*}),
                syn::Fields::Unnamed(_) => quote!((#(#read_fields),*)),
                _ => {
                    return Err(syn::Error::new_spanned(
                        &data.fields,
                        "Unit structs unsupported",
                    ))
                }
            };

            quote! {
                impl #generics crate::proto::ProtoType for #name #generic_names {
                    const CHECKSUM: u128 = {
                        let mut output = #ident_hash;
                        output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                        #(#cksum_fields)*
                        output
                    };
                }

                #client_source_cfg
                #server_source_cfg
                impl #generics crate::proto::BinWrite for #name #generic_names {
                    fn write(&self, buf: &mut Vec<u8>) {
                        #(#write_fields)*
                    }
                }

                #client_sink_cfg
                #server_sink_cfg
                impl #generics crate::proto::BinRead for #name #generic_names {
                    fn read(buf: &mut &[u8]) -> Result<Self, crate::proto::Error> {
                        Ok(Self #read_fields)
                    }
                }
            }
        }
        syn::Data::Enum(data) => {
            let discrim_ty = match data.variants.len() {
                n if n >= 256 => quote!(u16),
                _ => quote!(u8),
            };

            let mut write_variants = Vec::with_capacity(data.variants.len());
            let mut read_variants = Vec::with_capacity(data.variants.len());
            let mut cksum_variants = Vec::with_capacity(data.variants.len());
            for (i, variant) in data.variants.iter().enumerate() {
                let i = proc_macro2::Literal::usize_unsuffixed(i);

                let variant_name = &variant.ident;
                let variant_hash = hash(variant_name);
                let variant_ty = variant.fields.iter().next();
                let variant_ty_cksum = match &variant_ty {
                    Some(ty) => {
                        quote!(<#ty as crate::proto::ProtoType>::CHECKSUM)
                    }
                    None => quote!((1 << 64) - 59),
                };

                let write_value = variant_ty.map(|_| quote!((value)));
                let write_inner = quote!(crate::proto::BinWrite::write(value, &mut *buf););
                let write_inner = variant_ty.map(|_| write_inner);
                let read_inner =
                    variant_ty.map(|_| quote!((crate::proto::BinRead::read(&mut *buf)?)));

                write_variants.push(quote! {
                    Self::#variant_name #write_value => {
                        <#discrim_ty as crate::proto::BinWrite>::write(&#i, &mut *buf);
                        #write_inner
                    }
                });
                read_variants.push(quote! {
                    #i => Ok(Self::#variant_name #read_inner)
                });
                cksum_variants.push(quote! {
                    output = output.wrapping_add(#variant_hash);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);

                    output = output.wrapping_add(#variant_ty_cksum);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                });
            }

            quote! {
                impl #generics crate::proto::ProtoType for #name #generic_names {
                    const CHECKSUM: u128 = {
                        let mut output = #ident_hash;
                        output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                        #(#cksum_variants)*
                        output
                    };
                }

                #client_source_cfg
                #server_source_cfg
                impl #generics crate::proto::BinWrite for #name #generic_names {
                    fn write(&self, buf: &mut Vec<u8>) {
                        match self {
                            #(#write_variants,)*
                        }
                    }
                }

                #client_sink_cfg
                #server_sink_cfg
                impl #generics crate::proto::BinRead for #name #generic_names {
                    fn read(buf: &mut &[u8]) -> Result<Self, crate::proto::Error> {
                        let discrim = <#discrim_ty as crate::proto::BinRead>::read(&mut *buf)?;
                        match discrim {
                            #(#read_variants,)*
                            _ => Err(crate::proto::Error("Invalid discriminant".into())),
                        }
                    }
                }
            }
        }
        _ => return Err(syn::Error::new_spanned(input, "unions unsupported")),
    })
}

#[proc_macro_attribute]
pub fn system(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    system_imp(attr.into(), input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn system_imp(_system_attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<syn::ItemFn>(input)?;
    let name = &input.sig.ident;
    let body = &input.block;

    let mut attrs: Vec<_> = input.attrs.iter().collect();
    let thread_local = attrs
        .drain_filter(|attr| attr.path.is_ident("thread_local"))
        .count()
        > 0;
    let setup_method = if thread_local {
        quote!(system_local)
    } else {
        quote!(system)
    };

    let system_name = format_ident!("{}_system", name);
    let setup_name = format_ident!("{}_setup", name);

    let mut out_args = Vec::new();
    let mut state_values = Vec::new();
    let mut assigns = Vec::new();
    let mut var_adapters = Vec::new();

    let mut perf_name = None;

    for arg in &input.sig.inputs {
        let syn::PatType {
            pat,
            ty,
            attrs: arg_attrs,
            ..
        } = match arg {
            syn::FnArg::Receiver(recv) => {
                return Err(syn::Error::new_spanned(recv, "receiver not allowed"))
            }
            syn::FnArg::Typed(typed) => typed,
        };

        let mut arg_attrs: Vec<_> = arg_attrs.iter().collect();

        if let Some(attr) = arg_attrs.iter().find(|attr| attr.path.is_ident("resource")) {
            let mut is_perf = false;
            if let syn::Type::Reference(ty) = &**ty {
                if let syn::Type::Path(path) = &*ty.elem {
                    if path
                        .path
                        .segments
                        .iter()
                        .map(|segment| &segment.ident)
                        .collect::<Vec<_>>()
                        == ["codegen", "Perf"]
                    {
                        is_perf = true;
                    }
                }
            }
            if is_perf {
                perf_name = Some(quote!(#pat));
            } else {
                let mut no_init = false;
                let args = attr.parse_meta()?;
                if let syn::Meta::List(list) = &args {
                    for arg in &list.nested {
                        if let syn::NestedMeta::Meta(meta) = arg {
                            no_init =
                                matches!(meta, syn::Meta::Path(path) if path.is_ident("no_init"));
                        }
                    }
                }
                if !no_init {
                    if let syn::Type::Reference(ty) = &**ty {
                        let elem = &*ty.elem;
                        assigns.push(quote! {
                                setup = setup.resource(<#elem>::default());
                        });
                    }
                }
            }
        }

        if let Some(attr) = arg_attrs
            .drain_filter(|attr| attr.path.is_ident("state"))
            .next()
        {
            let expr = attr.parse_args::<syn::Expr>()?;
            state_values.push(quote!(#expr));
            out_args.push((quote!(#[state]), quote!(#pat), quote!(#ty)));
            continue;
        }

        if arg_attrs
            .drain_filter(|attr| attr.path.is_ident("subscriber"))
            .next()
            .is_some()
        {
            let event = match &**ty {
                syn::Type::ImplTrait(it) if it.bounds.len() == 1 => &it.bounds[0],
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "subscriber must have type `impl Iterator<Item = EventType>`",
                    ))
                }
            };
            let event = match event {
                syn::TypeParamBound::Trait(tb) => {
                    &tb.path.segments.last().expect("empty path").arguments
                }
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "subscriber must have type `impl Iterator<Item = EventType>`",
                    ))
                }
            };
            let event = match event {
                syn::PathArguments::AngleBracketed(ab) => ab.args.first().expect("empty <>"),
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "subscriber must have type `impl Iterator<Item = EventType>`",
                    ))
                }
            };
            let event = match event {
                syn::GenericArgument::Binding(bind) => bind,
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "subscriber must have type `impl Iterator<Item = EventType>`",
                    ))
                }
            };
            if event.ident != "Item" {
                return Err(syn::Error::new_spanned(
                    event,
                    "subscriber must have type `impl Iterator<Item = EventType>`",
                ));
            }
            let event = &event.ty;

            let pat = match &**pat {
                syn::Pat::Ident(ident) => &ident.ident,
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "subscriber must have simple variable name",
                    ))
                }
            };
            let reader_var_name = format_ident!("__reader_id_for_{}", &pat);
            let channel_var_name = format_ident!("__channel_for_{}", &pat);
            out_args.push((
                quote!(#[state]),
                quote!(#reader_var_name),
                quote!(&mut ::shrev::ReaderId<#event>),
            ));
            out_args.push((
                quote!(#[resource]),
                quote!(#channel_var_name),
                quote!(&::shrev::EventChannel<#event>),
            ));
            state_values.push(quote!(#reader_var_name));
            assigns.push(quote! {
                let #reader_var_name = setup.subscribe::<#event>();
            });
            var_adapters.push(quote! {
                let #pat = #channel_var_name.read(#reader_var_name);
            });
            continue;
        }

        out_args.push((quote!(#(#arg_attrs)*), quote!(#pat), quote!(#ty)));
    }

    let perf_name = match perf_name {
        Some(name) => name,
        None => {
            out_args.push((
                quote!(#[resource]),
                quote!(__traffloat_codegen_perf),
                quote!(&::codegen::Perf),
            ));
            quote!(__traffloat_codegen_perf)
        }
    };

    let out_arg_attrs: Vec<_> = out_args.iter().map(|tuple| &tuple.0).collect();
    let out_arg_names: Vec<_> = out_args.iter().map(|tuple| &tuple.1).collect();
    let out_arg_types: Vec<_> = out_args.iter().map(|tuple| &tuple.2).collect();

    let output = quote! {
        #[::legion::system]
        #[allow(clippy::too_many_arguments)]
        #(#attrs)*
        fn #name(
            #(#out_arg_attrs #out_arg_names: #out_arg_types),*
        ) {
            fn imp(#(#out_arg_names: #out_arg_types),*) {
                #(#var_adapters)*
                #body
            }

            let __traffloat_codegen_perf_start = ::codegen::hrtime();
            imp(#(#out_arg_names),*);
            let __traffloat_codegen_perf_end = ::codegen::hrtime();
            #perf_name.push(
                concat!(module_path!(), "::", stringify!(#name)),
                __traffloat_codegen_perf_end - __traffloat_codegen_perf_start,
            )
        }

        fn #setup_name(mut setup: ::codegen::SetupEcs) -> ::codegen::SetupEcs {
            #(#assigns)*
            setup
                .#setup_method(#system_name(#(#state_values),*))
        }
    };
    Ok(output)
}

fn hash(name: &syn::Ident) -> u128 {
    let mut hasher = crc64fast::Digest::new();
    hasher.write(name.to_string().as_bytes());
    hasher.sum64().into()
}
