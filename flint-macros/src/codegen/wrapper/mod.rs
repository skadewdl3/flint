use proc_macro2::TokenStream;
use quote::quote;
pub fn get_conditional_wrapper() -> TokenStream {
    let contents = include_str!("./conditional.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

pub fn get_iter_layout_wrapper() -> TokenStream {
    let contents = include_str!("./iter_layout.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

pub fn get_layout_wrapper() -> TokenStream {
    let contents = include_str!("./layout.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}

pub fn get_stateful_wrapper() -> TokenStream {
    let contents = include_str!("./stateful.rs");
    let contents = syn::parse_str::<TokenStream>(contents).unwrap();
    quote! {
        #contents
    }
}
