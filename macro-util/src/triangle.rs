use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

pub fn pm2(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<Input>(input)?;

    let mut inner = input.prefix.to_token_stream();
    for (i, pair) in input.exprs.into_pairs().enumerate() {
        input.repeater_paren.surround(&mut inner, |repeater| {
            repeater.extend((0..i).map(|_| input.repeater.clone()));
        });

        pair.value().to_tokens(&mut inner);
        pair.punct().to_tokens(&mut inner);
    }

    let target_macro = input.target_macro;
    Ok(quote::quote!(#target_macro! { #inner }))
}

struct Input {
    target_macro:   syn::Path,
    repeater_paren: syn::token::Paren,
    repeater:       TokenStream,
    prefix:         TokenStream,
    exprs:          Punctuated<TokenStream, syn::Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target_macro: syn::Path = input.parse()?;
        let repeater_parse;
        let repeater_paren = syn::parenthesized!(repeater_parse in input);
        let repeater: TokenStream = repeater_parse.parse()?;
        input.parse::<syn::Token![;]>()?;

        let mut prefix = TokenStream::new();
        while !input.peek(syn::Token![;]) {
            input.parse::<TokenTree>()?.to_tokens(&mut prefix);
        }
        input.parse::<syn::Token![;]>()?.to_tokens(&mut prefix);

        let exprs = input.parse_terminated(
            |inner| {
                let mut expr = TokenStream::new();
                while !inner.peek(syn::Token![,]) {
                    inner.parse::<TokenTree>()?.to_tokens(&mut expr);
                }
                Ok(expr)
            },
            syn::Token![,],
        )?;

        Ok(Self { target_macro, repeater_paren, repeater, prefix, exprs })
    }
}
