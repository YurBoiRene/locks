use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};

/// Creates a main handle
///
/// Wrap main like so.
/// This creates a `MainLevel` handle named `main`.
///
/// ```
/// #[locks::main]
/// fn main() {
///     // Use main here
/// }
/// ```
///
/// You can rename the handle by passing in an identifier.
///
/// ```
/// #[locks::main(main_hdl)]
/// fn main() {
///     // Use main_hdl here
/// }
///
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("Expected function."),
    };

    // Parse main handle identifier, defaults to `main`
    let main_arg = syn::parse::<Option<syn::Ident>>(_args)
        .expect("Expected identifier.")
        .unwrap_or(syn::Ident::new("main", Span::call_site()));

    // Insert code to get main handle first ("unsafe")
    fn_item.block.stmts.insert(
        0,
        syn::parse(quote!(let #main_arg = &mut unsafe { Handle::new(&MainLevel) };).into())
            .unwrap(),
    );

    item.into_token_stream().into()
}
