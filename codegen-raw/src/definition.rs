use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse, Error, Result};

pub(crate) fn imp(input: TokenStream) -> Result<TokenStream> {
    let cfg_gate = quote!(#[cfg(feature = "convert-human-friendly")]);

    let input = syn::parse2::<syn::DeriveInput>(input)?;
    let input_ident = &input.ident;

    let human_friendly_ident = format_ident!("{}HumanFriendly", input_ident);

    let (generics_bounded, generics_unbounded) = if input.generics.params.is_empty() {
        (quote!(), quote!())
    } else {
        (
            {
                let params = &input.generics.params;
                quote!(<#params>)
            },
            {
                let idents = input.generics.params.iter().map(|param| match param {
                    syn::GenericParam::Type(ty) => ty.ident.to_token_stream(),
                    syn::GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
                    syn::GenericParam::Const(cons) => cons.ident.to_token_stream(),
                });
                quote!(<#(#idents),*>)
            },
        )
    };
    let generics_where = &input.generics.where_clause;

    let hf_serde: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("hf_serde"))
        .map(|attr| &attr.tokens)
        .collect();

    let mut context_types = Vec::new();
    for attr in &input.attrs {
        if attr.path.is_ident("resolve_context") {
            let args = syn::parse2::<ResolveContextTypeList>(attr.tokens.clone())?;
            context_types.extend(args.0.into_iter());
        }
    }

    let need_id: bool;
    let human_friendly: TokenStream;
    let human_friendly_conversion: TokenStream;

    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => {
                let fields: Vec<_> = named
                    .named
                    .iter()
                    .filter(|field| field.attrs.iter().all(|attr| !attr.path.is_ident("hf_skip")))
                    .collect();

                need_id = fields.iter().any(|field| {
                    if let syn::Type::Path(path) = &field.ty {
                        if path.path.is_ident("Id") {
                            if let Some(ident) = &field.ident {
                                if ident == "id" {
                                    return true;
                                }
                            }
                        }
                    }
                    false
                });

                let mut field_idents = Vec::new();
                let mut field_idents_conv = Vec::new();
                let mut field_hf_serde = Vec::new();
                let mut field_conversion_ty = Vec::new();
                let mut field_conversion_expr = Vec::new();

                for field in &fields {
                    let field_ty = &field.ty;
                    let field_ident = field.ident.as_ref().expect("named struct");

                    if field_ident == "id_str" {
                        field_idents_conv.push(field_ident);
                        field_conversion_expr.push(quote! {
                            ::codegen::IdStr::new(human_friendly.id.clone())
                        });
                        continue;
                    }

                    field_idents.push(field_ident);
                    field_idents_conv.push(field_ident);
                    field_hf_serde.push(
                        field
                            .attrs
                            .iter()
                            .filter(|attr| attr.path.is_ident("hf_serde"))
                            .map(|attr| {
                                let tokens = &attr.tokens;
                                quote!(#[serde #tokens])
                            })
                            .collect::<TokenStream>(),
                    );
                    field_conversion_ty
                        .push(quote!(<#field_ty as ::codegen::Definition>::HumanFriendly));

                    let clone_call = if field_ident == "id" { quote!(.clone()) } else { quote!() };
                    field_conversion_expr.push(quote! {
                        <#field_ty as ::codegen::Definition>::convert(
                            human_friendly.#field_ident #clone_call,
                            context,
                        )?
                    });
                }

                let hf_skip = named
                    .named
                    .iter()
                    .filter(|field| field.attrs.iter().any(|attr| attr.path.is_ident("hf_skip")))
                    .map(|field| {
                        let field_ident = &field.ident;
                        let field_ty = &field.ty;
                        quote! {
                            #field_ident: <#field_ty as ::std::default::Default>::default(),
                        }
                    });

                human_friendly = quote! {
                    #cfg_gate
                    #[doc = concat!("The human-friendly version of [`", stringify!(#input_ident), "`].")]
                    #[derive(::serde::Serialize, ::serde::Deserialize)]
                    #(#[serde #hf_serde])*
                    pub struct #human_friendly_ident #generics_bounded #generics_where {
                        #(
                            #field_hf_serde
                            pub(crate) #field_idents: #field_conversion_ty,
                        )*
                    }

                    #cfg_gate
                    impl #generics_bounded #human_friendly_ident #generics_unbounded #generics_where {
                        #(
                            #[doc = concat!("See [`", stringify!(#input_ident), "::", stringify!(#field_idents), "`]")]
                            pub fn #field_idents(&self) -> &#field_conversion_ty {
                                &self.#field_idents
                            }
                        )*
                    }
                };
                human_friendly_conversion = quote! {
                    Self {
                        #(
                            #field_idents_conv: #field_conversion_expr,
                        )*
                        #(#hf_skip)*
                    }
                };
            }
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "derive(Definition) is only allowed on enums and named structs",
                ))
            }
        },
        syn::Data::Enum(e) => {
            need_id = false;

            let (variant_defs, variant_conversions): (Vec<_>, Vec<_>) = e.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let doc = quote!(#[doc = concat!("See [`", stringify!(#input_ident), "::", stringify!(#variant_ident), "`]")]);

                match &variant.fields {
                    syn::Fields::Unit => {
                        (
                            quote!(#doc #variant_ident),
                            quote! {
                                #human_friendly_ident::#variant_ident => Self::#variant_ident
                            },
                        )
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let (field_conversion_ty, field_conversion_expr): (Vec<_>, Vec<_>) = unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(ord, field)| {
                                let field_ty = &field.ty;
                                let field_ident = format_ident!("_field_{}", ord);

                                (
                                    quote!(<#field_ty as ::codegen::Definition>::HumanFriendly),
                                    quote!(
                                        <#field_ty as ::codegen::Definition>::convert(
                                            #field_ident,
                                            context,
                                        )?
                                    ),
                                )
                            })
                            .unzip();

                        let field_idents = (0..(unnamed.unnamed.len())).map(|ord| format_ident!("_field_{}", ord));

                        (
                            quote!(#doc #variant_ident(#(#field_conversion_ty),*)),
                            quote! {
                                #human_friendly_ident::#variant_ident(#(#field_idents),*) => Self::#variant_ident(#(#field_conversion_expr),*)
                            },
                        )
                    }
                    syn::Fields::Named(named) => {
                        let field_idents: Vec<_> = named.named.iter().map(|field| &field.ident).collect();

                        let (field_conversion_ty, field_conversion_expr): (Vec<_>, Vec<_>) = named
                            .named
                            .iter()
                            .map(|field| {
                                let field_ty = &field.ty;
                                let field_ident = &field.ident;

                                (
                                    quote!(<#field_ty as ::codegen::Definition>::HumanFriendly),
                                    quote!(
                                        <#field_ty as ::codegen::Definition>::convert(
                                            #field_ident,
                                            context,
                                        )?
                                    ),
                                )
                            })
                            .unzip();

                        (
                            quote!(#doc #variant_ident {
                                #(#doc #field_idents: #field_conversion_ty),*
                            }),
                            quote! {
                                #human_friendly_ident::#variant_ident{#(#field_idents),*} => Self::#variant_ident{
                                    #( #field_idents: #field_conversion_expr),*
                                }
                            },
                        )
                    }
                }
            }).unzip();

            human_friendly = quote! {
                #cfg_gate
                #[doc = concat!("The human-friendly version of [`", stringify!(#input_ident), "`].")]
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(tag = "type")]
                #(#[serde #hf_serde])*
                pub enum #human_friendly_ident #generics_bounded #generics_where {
                    #(#variant_defs),*
                }
            };

            human_friendly_conversion = quote! {
                match human_friendly {
                    #(#variant_conversions),*
                }
            };
        }
        syn::Data::Union(_) => {
            return Err(Error::new(
                Span::call_site(),
                "derive(Definition) is only allowed on enums and named structs",
            ))
        }
    }

    let id = need_id.then(|| quote! {
        #[doc = stringify!("An ordinal runtime ID for [`", stringify!(#input_ident), "`].")]
        #[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Id(u32);

        impl Id {
            /// Use this ID as a key to index a Vec.
            pub fn as_index(&self) -> usize {
                use ::std::convert::TryInto;

                self.0.try_into().expect("Too many items")
            }
        }

        #cfg_gate
        impl ::codegen::Definition for Id {
            type HumanFriendly = ::arcstr::ArcStr;

            fn convert(human_friendly: Self::HumanFriendly, context: &mut ::codegen::ResolveContext) -> ::anyhow::Result<Self> {
                use ::std::convert::TryFrom;

                use ::anyhow::Context as _;

                // only #input_ident is used here because generic types are not allowed to have
                // their own IDs.
                let id = context.resolve_id::<#input_ident>(human_friendly.as_str())?;
                let id = u32::try_from(id).context("Too many items")?;
                Ok(Self(id))
            }
        }

        impl ::codegen::Identifier for Id {
            type Def = #input_ident; // identifiable types can't have generics

            fn index(self, list: &[#input_ident]) -> Option<&Self::Def> {
                list.get(self.as_index())
            }
        }

        impl #generics_bounded ::codegen::Identifiable for #input_ident #generics_unbounded #generics_where {
            type Id = Id;

            fn id(&self) -> Id {
                self.id
            }

            fn id_str(&self) -> &IdStr {
                &self.id_str
            }
        }
    });

    let register_id = need_id.then(|| {
        quote! {
            context.notify::<#input_ident>(human_friendly.id.clone())?;
        }
    });

    let context_setup = context_types
        .iter()
        .map(|ty| {
            quote! {
                context.start_tracking::<#ty>();
            }
        })
        .collect::<TokenStream>();
    let context_shutdown = context_types
        .iter()
        .map(|ty| {
            quote! {
                context.stop_tracking::<#ty>();
            }
        })
        .collect::<TokenStream>();

    let post_convert: TokenStream = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("hf_post_convert"))
        .map(|attr| syn::parse2::<PostConvert>(attr.tokens.clone()))
        .map(|result| {
            result.map(|pc| {
                let path = &pc.0;
                quote!(#path(&mut ret, context)?;)
            })
        })
        .collect::<syn::Result<_>>()?;

    let ret_mut = (!post_convert.is_empty()).then(|| quote!(mut));

    let output = quote! {
        #id

        #human_friendly

        #cfg_gate
        impl #generics_bounded ::codegen::Definition for #input_ident #generics_unbounded #generics_where {
            type HumanFriendly = #human_friendly_ident #generics_unbounded;

            fn convert(human_friendly: #human_friendly_ident #generics_unbounded, context: &mut ::codegen::ResolveContext) -> ::anyhow::Result<Self> {
                #context_setup

                #register_id
                context.trigger_listener::<#input_ident #generics_unbounded>(&human_friendly)?;

                let #ret_mut ret = #human_friendly_conversion;

                #post_convert

                #context_shutdown

                Ok(ret)
            }
        }
    };

    Ok(output)
}

struct ResolveContextTypeList(Punctuated<syn::Type, syn::Token![,]>);

impl parse::Parse for ResolveContextTypeList {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let inner;
        syn::parenthesized!(inner in input);
        let list = Punctuated::parse_terminated(&inner)?;
        Ok(Self(list))
    }
}

struct PostConvert(syn::Path);

impl parse::Parse for PostConvert {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let inner;
        syn::parenthesized!(inner in input);
        let list = inner.parse()?;
        Ok(Self(list))
    }
}
