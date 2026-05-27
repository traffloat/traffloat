use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

/// Repeats a delimited stream with increasing frequencies of a suffix token.
///
/// # Syntax
/// The following tokens are expected in order:
/// - Target: A `syn::Path` for the target macro to invoke
/// - Repeater: A parenthesized group of tokens to repeat
/// - Prefix: Zero or multiple non-semicolon tokens (except in token trees), followed by a semicolon
/// - Expressions: Token streams delimited by commas, with optional trailing comma.
///   Each expression may be zero or multiple non-comma tokens (except in token trees).
///
/// # Example
/// ```
/// # macro_rules! target_macro {
/// #   (
/// #       prefix 1, prefix 2;
/// #       () expr 1,
/// #       (foo) expr 2,
/// #       (foo foo) expr 3,
/// #       (foo foo foo) expr 4,
/// #   ) => {}
/// # }
///
/// traffloat_macro_util::triangle! {
///     target_macro (foo);
///     prefix 1, prefix 2;
///     expr 1,
///     expr 2,
///     expr 3,
///     expr 4,
/// }
///
/// // expands into
///
/// target_macro! {
///     prefix 1, prefix 2;
///     () expr 1,
///     (foo) expr 2,
///     (foo foo) expr 3,
///     (foo foo foo) expr 4,
/// }
/// ```
#[proc_macro]
pub fn triangle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    triangle_pm2(input.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn triangle_pm2(input: TokenStream) -> syn::Result<TokenStream> {
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
