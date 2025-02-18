use crate::{
    widget::{Widget, WidgetRenderer},
    MacroInput,
};

use super::{util::get_render_function, WidgetHandlerOptions};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Handles the rendering of a widget stored in a variable.
/// Sometimes, we need to render a widget constructed elsewhere. That's where this function comes in.
/// We can use the {{ my_widget }} syntax to render a widget stored in a variable.
/// This function takes an expression representing the widget to be rendered and configuration options for widget handling.
///
///
/// The {{  }} syntax accept any valid expression that implements the ratatui::Widget trait.
/// So, you can use function calls that return a widget, if-else expressions, etc.
///
/// # Arguments
///
/// * `variable` - The expression representing the widget to be rendered
/// * `options` - Configuration options for widget handling
///
/// # Returns
///
/// A TokenStream containing either a render_widget call (if top level) or just the variable reference
pub fn handle_variable_widget(
    widget: &Widget,
    variable: &Expr,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        input,
        ..
    } = options;

    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);
        if *is_top_level {
            match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    #render_fn(#render_ref_code #variable, #area, #buffer);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame .#frame_render_fn(#render_ref_code #variable, #frame.area());
                },
            }
        } else {
            quote! { #variable }
        }
    } else {
        quote! { #variable }
    }
}
