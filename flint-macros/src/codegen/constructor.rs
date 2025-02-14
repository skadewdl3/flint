use crate::{arg::ArgKind, widget::Widget};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::WidgetHandlerOptions;

/// Handles the generation of widget construction code. This is the simplest kind of widget.
/// It's called a constructor widget since we can specify the constructor function to use
/// as well as any additional arguments required for the widget's construction.
///
/// This function takes a widget definition and generates the appropriate TokenStream
/// for constructing that widget, including both positional and named arguments.
///
/// # Arguments
///
/// * `widget` - The widget definition containing arguments and configuration
/// * `name` - The identifier for the widget type/name
/// * `constructor` - The identifier for the widget's constructor function
/// * `options` - Additional options controlling widget generation behavior
///
/// # Returns
///
/// Returns a TokenStream containing the widget construction code. If the widget is
/// marked as top-level, the code will include rendering the widget to a frame.
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
