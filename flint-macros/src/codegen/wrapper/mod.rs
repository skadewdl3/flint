use proc_macro2::TokenStream;
use quote::quote;

/// Retrieves and parses the contents of `conditional.rs` into a TokenStream.
///
/// This wrapper function includes the conditional layout implementation code
/// and returns it as a parsed and quoted TokenStream ready for macro expansion.
pub fn get_conditional_wrapper() -> TokenStream {
    let contents = include_str!("./conditional.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

/// Retrieves and parses the contents of `iter_layout.rs` into a TokenStream.
///
/// This wrapper function includes the iterator-based layout implementation code
/// and returns it as a parsed and quoted TokenStream ready for macro expansion.
pub fn get_iter_layout_wrapper() -> TokenStream {
    let contents = include_str!("./iter_layout.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

/// Retrieves and parses the contents of `layout.rs` into a TokenStream.
///
/// This wrapper function includes the basic layout implementation code
/// and returns it as a parsed and quoted TokenStream ready for macro expansion.
pub fn get_layout_wrapper() -> TokenStream {
    let contents = include_str!("./layout.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

/// Retrieves and parses the contents of `stateful.rs` into a TokenStream.
///
/// This wrapper function includes the stateful component implementation code
/// and returns it as a parsed and quoted TokenStream ready for macro expansion.
pub fn get_stateful_wrapper() -> TokenStream {
    let contents = include_str!("./stateful.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}
