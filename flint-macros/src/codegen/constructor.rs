use crate::{arg::ArgKind, widget::Widget};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::WidgetHandlerOptions;

pub fn handle_constructor_widget(
    widget: &Widget,
    name: &Ident,
    constructor: &Ident,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        frame,
        ..
    } = options;

    let args = &widget.args;

    let positional_args: Vec<_> = args
        .iter()
        .filter_map(|arg| match &arg.kind {
            ArgKind::Positional => Some(&arg.value),
            _ => None,
        })
        .collect();

    // Start with constructor call including all positional arguments
    let mut widget = quote! {
        #name :: #constructor(#(#positional_args),*)
    };

    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            widget.extend(quote! {
                .#name(#value)
            });
        }
    }

    if *is_top_level {
        quote! {
            #frame .render_widget(#widget, #frame.area());
        }
    } else {
        widget
    }
}
