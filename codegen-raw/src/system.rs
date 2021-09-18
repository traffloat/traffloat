use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse;
use syn::Result;

pub(crate) fn imp(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2::<AttrArgs>(args)?;

    let input = syn::parse2::<syn::ItemFn>(input)?;
    let name = &input.sig.ident;
    let body = &input.block;

    let mut attrs: Vec<_> = input.attrs.iter().collect();
    let thread_local = attrs.drain_filter(|attr| attr.path.is_ident("thread_local")).count() > 0;
    let setup_method = if thread_local { quote!(system_local) } else { quote!(system) };

    let system_name = format_ident!("{}_system", name);
    let setup_name = format_ident!("{}_setup", name);

    let mut out_args = Vec::new();
    let mut state_values = Vec::new();
    let mut assigns = Vec::new();
    let mut var_adapters = Vec::new();

    let mut perf_name = None;

    let mut has_debug_entries_resource = false;

    'param_loop: for arg in &input.sig.inputs {
        let syn::PatType { pat, ty, attrs: arg_attrs, .. } = match arg {
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
                    if path.path.segments.iter().map(|segment| &segment.ident).collect::<Vec<_>>()
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
                            setup = setup.resource_default::<#elem>();
                        });
                    }
                }
            }
        }

        if let Some(attr) = arg_attrs.drain_filter(|attr| attr.path.is_ident("state")).next() {
            let expr = attr.parse_args::<syn::Expr>()?;
            state_values.push(quote!(#expr));
            out_args.push((quote!(#[state]), quote!(#pat), quote!(#ty)));
            continue;
        }

        for attr in &arg_attrs {
            if attr.path.is_ident("debug") {
                if cfg!(feature = "render-debug") {
                    let DebugName { category, name } =
                        syn::parse2::<DebugName>(attr.tokens.clone())?;
                    if !has_debug_entries_resource {
                        assigns.push(quote! {
                            let mut __debug_entries = setup.resources.get_mut_or_default::<codegen::DebugEntries>();
                        });
                        has_debug_entries_resource = true;
                    }

                    let entry_name = format_ident!(
                        "__debug_entry_for_{}",
                        match &**pat {
                            syn::Pat::Ident(ident) => &ident.ident,
                            span => {
                                return Err(syn::Error::new_spanned(
                                    span,
                                    "debug entry must have simple variable name",
                                ));
                            }
                        }
                    );
                    assigns.push(quote! {
                        let #entry_name = __debug_entries.entry(#category, #name).clone();
                    });
                    state_values.push(quote!(#entry_name));
                    out_args.push((quote!(#[state]), quote!(#pat), quote!(#ty)));
                } else {
                    let raw = match &**ty {
                        syn::Type::Reference(ty) => &ty.elem,
                        ty => {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "debug entry must have type &mut DebugEntry",
                            ))
                        }
                    };
                    state_values.push(quote!(#raw(())));
                    out_args.push((quote!(#[state]), quote!(#pat), quote!(#ty)))
                }
                continue 'param_loop;
            }
        }

        if arg_attrs.drain_filter(|attr| attr.path.is_ident("subscriber")).next().is_some() {
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
                let #reader_var_name = setup.subscriber::<#event>();
            });
            var_adapters.push(quote! {
                let mut #pat = #channel_var_name.read(#reader_var_name);
            });
            continue;
        }

        if arg_attrs.drain_filter(|attr| attr.path.is_ident("publisher")).next().is_some() {
            let event = match &**ty {
                syn::Type::ImplTrait(it) if it.bounds.len() == 1 => &it.bounds[0],
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "publisher must have type `impl FnMut(EventType)`",
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
                        "publisher must have type `impl FnMut(EventType)`",
                    ))
                }
            };
            let event = match event {
                syn::PathArguments::Parenthesized(ab) => &ab.inputs,
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "publisher must have type `impl FnMut(EventType)`",
                    ))
                }
            };
            let event = match event.first() {
                Some(event) => event,
                None => {
                    return Err(syn::Error::new_spanned(
                        event,
                        "publisher must have type `impl FnMut(EventType)`",
                    ))
                }
            };

            let pat = match &**pat {
                syn::Pat::Ident(ident) => &ident.ident,
                span => {
                    return Err(syn::Error::new_spanned(
                        span,
                        "publisher must have simple variable name",
                    ))
                }
            };
            out_args.push((
                quote!(#[resource]),
                quote!(#pat),
                quote!(&mut ::shrev::EventChannel<#event>),
            ));
            assigns.push(quote! {
                setup = setup.resource_default::<::shrev::EventChannel<#event>>();
            });
            var_adapters.push(quote! {
                let mut #pat = move |event| #pat.single_write(event);
            });
            continue;
        }

        out_args.push((quote!(#(#arg_attrs)*), quote!(#pat), quote!(#ty)));
    }

    if has_debug_entries_resource {
        assigns.push(quote! {
            drop(__debug_entries);
        });
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

    let class = &args.class;

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
            setup.#setup_method(#system_name(#(#state_values),*), ::codegen::SystemClass::#class)
        }
    };
    Ok(output)
}

struct DebugName {
    category: String,
    name: String,
}

impl parse::Parse for DebugName {
    fn parse(buf: parse::ParseStream) -> parse::Result<Self> {
        let inner;
        syn::parenthesized!(inner in buf);
        let category = inner.parse::<syn::LitStr>()?;
        inner.parse::<syn::Token![,]>()?;
        let name = inner.parse::<syn::LitStr>()?;
        Ok(Self { category: category.value(), name: name.value() })
    }
}

struct AttrArgs {
    class: Ident,
}

impl parse::Parse for AttrArgs {
    fn parse(buf: parse::ParseStream) -> parse::Result<Self> {
        let class = buf.parse::<Ident>()?;
        Ok(Self { class })
    }
}
