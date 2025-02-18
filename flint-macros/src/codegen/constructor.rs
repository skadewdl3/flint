use crate::{
    arg::ArgKind,
    widget::{util::get_render_method, Widget, WidgetRenderer},
    MacroInput,
};
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
        input,
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
    let mut widget_code = quote! {
        #name :: #constructor(#(#positional_args),*)
    };

    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            widget_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_method, frame_render_method) = get_render_method(widget);
        if *is_top_level {
            match renderer {
                WidgetRenderer::Area { area, buffer } => quote! {
                    #widget_code.#render_method(#area, #buffer)
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame.#frame_render_method(#widget_code, #frame.area());
                },
            }
        } else {
            widget_code
        }
    } else {
        widget_code
    }
}
