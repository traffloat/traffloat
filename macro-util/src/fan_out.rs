use std::iter::{self, Peekable};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

pub fn pm2(input: TokenStream) -> syn::Result<TokenStream> {
    let Input { prefix, target_macro, tuple_macro, item_macro, breadth, depth, patterns } =
        syn::parse2::<Input>(input)?;
    let breadth: u16 = breadth.base10_parse()?;
    let depth: u8 = depth.base10_parse()?;

    let recursive_tuples = generate_recursive_tuples(
        prefix.as_ref(),
        breadth,
        depth,
        &tuple_macro,
        patterns.iter().map(|Pattern { ident, tt }| quote!(#item_macro!(#prefix #ident #tt))),
    )?;

    let paths =
        generate_paths(breadth, depth).take(patterns.len()).collect::<syn::Result<Vec<_>>>()?;
    let patterns_with_paths = patterns.iter().zip(paths).map(|(pattern, path)| {
        let Pattern { ident, tt } = pattern;
        let path_idents = path.iter().map(|i| syn::Ident::new(&format!("p{i}"), ident.span()));
        quote! {
            #ident #tt (#(#path_idents)*),
        }
    });

    let ts = quote! {
        #target_macro! {
            #prefix
            #recursive_tuples;
            {
                #(#patterns_with_paths)*
            }
        }
    };
    Ok(ts)
}

fn generate_recursive_tuples(
    prefix: Option<&TokenTree>,
    breadth: u16,
    depth: u8,
    tuple_macro: &syn::Path,
    mut items: impl Iterator<Item = TokenStream>,
) -> syn::Result<TokenStream> {
    let out = generate_recursive_tuples_recurse(
        prefix,
        breadth,
        depth,
        tuple_macro,
        &mut items.by_ref().peekable(),
    )?;
    if items.next().is_some() {
        Err(syn::Error::new(Span::call_site(), "too many patterns for the given breadth and depth"))
    } else {
        Ok(out)
    }
}

fn generate_recursive_tuples_recurse(
    prefix: Option<&TokenTree>,
    breadth: u16,
    depth: u8,
    tuple_macro: &syn::Path,
    items: &mut Peekable<impl Iterator<Item = TokenStream>>,
) -> syn::Result<TokenStream> {
    if depth == 0 {
        return Err(syn::Error::new(Span::call_site(), "depth must be positive"));
    }

    if depth == 1 {
        let take_items = items.take(breadth as usize);
        Ok(quote!(#tuple_macro!(
            #prefix
            #(#take_items,)*
        )))
    } else {
        let mut out = TokenStream::new();
        for _ in 0..breadth {
            if items.peek().is_none() {
                break;
            }
            out.extend(generate_recursive_tuples_recurse(
                prefix,
                breadth,
                depth - 1,
                tuple_macro,
                items,
            )?);
            out.extend(quote!(,));
        }
        Ok(quote!(#tuple_macro!(#prefix #out)))
    }
}

fn generate_paths(breadth: u16, depth: u8) -> impl Iterator<Item = syn::Result<Vec<u16>>> {
    fn increment(buf: &mut [u16], breadth: u16, depth: u8) -> syn::Result<()> {
        if breadth == 0 {
            return Err(syn::Error::new(Span::call_site(), "breadth must be positive"));
        }

        let Some((last, init)) = buf.split_last_mut() else {
            return Err(syn::Error::new(
                Span::call_site(),
                format_args!("too many paths, only {} allowed", breadth.pow(depth.into())),
            ));
        };
        if *last < breadth - 1 {
            *last += 1;
        } else {
            increment(init, breadth, depth)?;
            *last = 0;
        }
        Ok(())
    }

    let mut buf = vec![0u16; depth as usize];
    iter::from_fn(move || {
        let result = buf.clone();
        Some(increment(&mut buf, breadth, depth).map(|()| result))
    })
}

struct Input {
    prefix:       Option<TokenTree>,
    target_macro: syn::Path,
    tuple_macro:  syn::Path,
    item_macro:   syn::Path,
    breadth:      syn::LitInt,
    depth:        syn::LitInt,
    patterns:     Punctuated<Pattern, syn::Token![,]>,
}

struct Pattern {
    ident: syn::Ident,
    tt:    TokenTree,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let prefix =
            if input.peek(syn::token::Bracket) { Some(input.parse::<TokenTree>()?) } else { None };

        let target_macro: syn::Path = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let tuple_macro: syn::Path = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let item_macro: syn::Path = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        let breadth: syn::LitInt = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let depth: syn::LitInt = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        let patterns = input.parse_terminated(
            |input| {
                let ident: syn::Ident = input.parse()?;
                let tt: TokenTree = input.parse()?;
                Ok(Pattern { ident, tt })
            },
            syn::Token![,],
        )?;

        Ok(Self { prefix, target_macro, tuple_macro, item_macro, breadth, depth, patterns })
    }
}
