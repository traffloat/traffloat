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
    triangle::pm2(input.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

mod triangle;

/// Fans out a pattern to tuples of size `$breadth` recursively with a depth of `$depth`,
/// allowing up to `$breadth`^`$depth` repetitions of the variant.
///
/// # Syntax
/// ```
/// # /*
/// fan_out! {
///     [$prefix]
///     $target_macro, $tuple_macro, $item_macro;
///     $breadth, $depth;
///     $ident0($args0),
///     $ident1($args1),
///     ...
/// }
/// # */
/// ```
///
/// Expands to:
///
/// ```
/// # /*
/// $target_macro! {
///     [$prefix]
///     $tuple_macro!(
///         [$prefix]
///         $tuple_macro!(
///                 [$prefix]
///             $tuple_macro!(
///                 [$prefix]
///                 // Recursive parentheses for $depth levels.
///                 // In this example, $depth = 3.
///                 $item_macro!([$prefix] $ident0($args0)),
///                 $item_macro!([$prefix] $ident1($args1)),
///                 ...
///                 // up to $breadth times
///             ),
///             // repeats as necessary
///         ),
///         // repeats as necessary
///     );
///     {
///         $ident0($args0) ($path0),
///         $ident1($args1) ($path1),
///         ...
///     }
/// }
/// # */
/// ```
/// `$pathN` is a token stream of `$depth` idents in the form `p{K}`,
/// where `0 <= K < $depth` forming a base-`$breadth` big-endian counter.
#[proc_macro]
pub fn fan_out(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fan_out::pm2(input.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

mod fan_out;
