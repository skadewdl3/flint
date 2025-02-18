use crate::{
    widget::{util::get_render_method, Widget, WidgetRenderer},
    MacroInput,
};

use super::WidgetHandlerOptions;
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

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_method, frame_render_method) = get_render_method(widget);

        if *is_top_level {
            match renderer {
                WidgetRenderer::Area { area, buffer } => quote! {
                    #variable.#render_method(#area, #buffer);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame.#frame_render_method(#variable, #frame.area());
                },
            }
        } else {
            quote! { #variable }
        }
    } else {
        quote! { #variable }
    }
}
