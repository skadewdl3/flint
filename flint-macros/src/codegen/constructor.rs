use crate::{
    arg::ArgKind,
    widget::{Widget, WidgetRenderer},
    MacroInput,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{util::get_render_function, WidgetHandlerOptions};

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
            if widget.stateful && name.to_string() == "state" {
                continue;
            }
            let value = &arg.value;
            widget_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    let stateful_code = match widget.stateful {
        true => {
            let (_, value) = args
                .iter()
                .filter_map(|arg| {
                    if let ArgKind::Named(ref ident) = arg.kind {
                        Some((ident, &arg.value))
                    } else {
                        None
                    }
                })
                .find(|(ident, _)| ident.to_string() == "state")
                .unwrap();

            quote! {
                , &mut #value
            }
        }
        false => quote! {},
    };

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);
        if *is_top_level {
            match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    #render_fn(#render_ref_code #widget_code, #area, #buffer #stateful_code);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame .#frame_render_fn(#render_ref_code #widget_code, #frame.area() #stateful_code);
                },
            }
        } else {
            widget_code
        }
    } else {
        widget_code
    }
}
