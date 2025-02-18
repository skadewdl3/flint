use super::Widget;
use proc_macro2::TokenStream;
use quote::quote;

pub fn get_render_method(widget: &Widget) -> (TokenStream, TokenStream) {
    let render_method = if widget.render_ref {
        quote! { render_ref }
    } else {
        quote! { render }
    };

    let frame_render_method = if widget.render_ref {
        quote! { render_widget_ref }
    } else {
        quote! { render_widget }
    };
    (render_method, frame_render_method)
}
