use super::{
    util::{generate_unique_id, get_render_function},
    WidgetHandlerOptions,
};
use crate::{
    widget::{Widget, WidgetRenderer},
    MacroInput,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ExprBlock;

/// Handles the rendering of a widget stored in a variable.
/// This function processes widgets that have been previously constructed and stored in variables.
/// The {{ my_widget }} macro syntax is used to render these stored widgets.
///
/// This handler serves two main purposes:
/// 1. For top-level widgets, it generates code to render the widget using the appropriate render function
/// 2. For nested widgets, it returns a reference to the widget variable
///
/// The {  } syntax can contain any valid Rust expression that returns a type implementing ratatui::Widget.
/// This includes:
/// - Direct variable references (e.g. {{ my_widget }})
/// - Function calls (e.g. {{ create_widget() }})
/// - If/else expressions (e.g. {{ if condition { widget1 } else { widget2 } }})
/// - Match expressions
/// - Any other expression returning anything that implements Widget/WidgetRef/StatefulWidget/StatefulWidgetRef
///
/// # Arguments
///
/// * `widget` - Contains widget metadata like whether to use references
/// * `variable` - The Rust expression block containing the widget to render
/// * `options` - Configuration including whether this is a top-level widget and rendering context
///
/// # Returns
///
/// A TokenStream that either:
/// - For top-level widgets: Generates a render_widget() call with the appropriate frame/buffer
/// - For nested widgets: Returns the widget variable reference for use in parent widgets
pub fn handle_variable_widget(
    widget: &Widget,
    variable: &ExprBlock,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    // Extract key options
    let WidgetHandlerOptions {
        is_top_level,
        input,
        ..
    } = options;

    // Determine if we need reference syntax
    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    // Generate unique identifier for the variable
    let variable_index = generate_unique_id() as usize;
    let variable_ident =
        proc_macro2::Ident::new(&format!("variable_{}", variable_index), Span::call_site());

    // Handle UI widget rendering
    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);
        if *is_top_level {
            return match renderer {
                // Render to specific area and buffer
                WidgetRenderer::Area { area, buffer } => quote! {
                    let #variable_ident = #variable;
                    #render_fn(#render_ref_code #variable_ident, #area, #buffer);
                },

                // Render to frame with computed area
                WidgetRenderer::Frame(frame) => quote! {
                    let #variable_ident = #variable;
                    #frame .#frame_render_fn(#render_ref_code #variable_ident, #frame.area());
                },
            };
        }
    }

    // For nested widgets, return the variable reference
    quote! {{ let #variable_ident = #variable; #variable_ident }}
}
