use super::{util::get_render_function, WidgetHandlerOptions};
use crate::{
    arg::ArgKind,
    widget::{Widget, WidgetRenderer},
    MacroInput,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Handles the generation of widget construction code. This is the simplest kind of widget.
/// It's called a constructor widget since we can specify the constructor function to use
/// as well as any additional arguments required for the widget's construction.
///
/// This function takes a widget definition and generates the appropriate TokenStream
/// for constructing that widget, including both positional and named arguments.
/// It handles three main tasks:
///
/// 1. Generates constructor call with positional arguments
/// 2. Adds any named arguments as method chaining
/// 3. If the widget is top-level, generates code to render it to a frame or area
///
/// # Arguments
///
/// * `widget` - The widget definition containing arguments and configuration, including
///             positional args, named args, and render settings
/// * `name` - The identifier for the widget type/name that will be constructed
/// * `constructor` - The identifier for the specific constructor function to call
/// * `options` - Additional options including whether this is a top-level widget
///              and what kind of UI input is being used
///
/// # Returns
///
/// Returns a TokenStream containing:
/// - Just the widget construction code for non-top-level widgets
/// - Construction + rendering code for top-level widgets, using either:
///   - Area rendering with area and buffer
///   - Frame rendering with frame.render() calls
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

    // Extract just the positional arguments in order
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

    // Add any named arguments as chained method calls
    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            widget_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    let (render_fn, frame_render_fn) = get_render_function(widget);

    // Determine if we need to pass widget by reference when rendering
    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    // For top-level widgets in UI context, generate rendering code
    if let MacroInput::Ui { renderer, .. } = input {
        if *is_top_level {
            return match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    #render_fn(#render_ref_code #widget_code, #area, #buffer);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame .#frame_render_fn(#render_ref_code #widget_code, #frame.area());
                },
            };
        }
    }

    // For non-top-level widgets, just return the construction code
    quote! {
        #widget_code
    }
}
