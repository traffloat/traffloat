extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Result;

#[proc_macro_derive(Gen, attributes(max_size, from_client_only, from_server_only))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    imp(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn imp(input: TokenStream) -> Result<TokenStream> {
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
                impl crate::proto::ProtoType for #name {
                    const CHECKSUM: u128 = {
                        let mut output = #ident_hash;
                        output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                        #(#cksum_fields)*
                        output
                    };
                }

                #client_source_cfg
                #server_source_cfg
                impl crate::proto::BinWrite for #name {
                    fn write(&self, buf: &mut Vec<u8>) {
                        #(#write_fields)*
                    }
                }

                #client_sink_cfg
                #server_sink_cfg
                impl crate::proto::BinRead for #name {
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
                impl crate::proto::ProtoType for #name {
                    const CHECKSUM: u128 = {
                        let mut output = #ident_hash;
                        output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                        #(#cksum_variants)*
                        output
                    };
                }

                #client_source_cfg
                #server_source_cfg
                impl crate::proto::BinWrite for #name {
                    fn write(&self, buf: &mut Vec<u8>) {
                        match self {
                            #(#write_variants,)*
                        }
                    }
                }

                #client_sink_cfg
                #server_sink_cfg
                impl crate::proto::BinRead for #name {
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

fn hash(name: &syn::Ident) -> u128 {
    let mut hasher = crc64fast::Digest::new();
    hasher.write(name.to_string().as_bytes());
    hasher.sum64().into()
}
