use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Error, Result};

pub(crate) fn imp(input: TokenStream) -> Result<TokenStream> {
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

    let need_id: bool;
    let human_friendly: TokenStream;
    let human_friendly_conversion: TokenStream;

    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => {
                need_id = named.named.iter().any(|field| {
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
                                    human_friendly.#field_ident,
                                    ::std::rc::Rc::clone(&resolve_name),
                                )?
                            ),
                        )
                    })
                    .unzip();

                human_friendly = quote! {
                    #[doc = concat!("The human-friendly version of [`", stringify!(#input_ident), "`].")]
                    #[derive(::serde::Serialize, ::serde::Deserialize)]
                    #(#[serde #hf_serde])*
                    pub struct #human_friendly_ident #generics_bounded #generics_where {
                        #(
                            #field_idents: #field_conversion_ty,
                        )*
                    }
                };
                human_friendly_conversion = quote! {
                    Self {
                        #(
                            #field_idents: #field_conversion_expr,
                        )*
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
                                            ::std::rc::Rc::clone(&resolve_name),
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
                                            ::std::rc::Rc::clone(&resolve_name),
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
        /// An ordinal runtime ID for 
        #[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Id(usize);

        impl ::codegen::Definition for Id {
            type HumanFriendly = ::arcstr::ArcStr;

            fn convert(human_friendly: Self::HumanFriendly, resolve_name: ::codegen::ResolveName) -> ::anyhow::Result<Self> {
                match resolve_name(human_friendly.as_str()) {
                    Some(id) => Ok(Self(id)),
                    None => ::anyhow::bail!("Cannot resolve name for {} ID: {}", stringify!(#input_ident), human_friendly.as_str()),
                }
            }
        }

        impl #generics_bounded ::codegen::Identifiable for #input_ident #generics_unbounded #generics_where {
            type Id = Id;

            fn id(&self) -> Id {
                self.id
            }
        }
    });

    let output = quote! {
        #id

        #human_friendly

        impl #generics_bounded ::codegen::Definition for #input_ident #generics_unbounded #generics_where {
            type HumanFriendly = #human_friendly_ident #generics_unbounded;

            fn convert(human_friendly: #human_friendly_ident #generics_unbounded, resolve_name: ::codegen::ResolveName) -> ::anyhow::Result<Self> {
                Ok(#human_friendly_conversion)
            }
        }
    };

    Ok(output)
}
