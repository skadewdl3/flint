use crate::{
    widget::{Widget, WidgetRenderer},
    MacroInput,
};

use super::{
    util::{generate_unique_id, get_render_function},
    WidgetHandlerOptions,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ExprBlock;

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
    variable: &ExprBlock,
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

    let variable_index = generate_unique_id() as usize;
    let variable_ident =
        proc_macro2::Ident::new(&format!("variable_{}", variable_index), Span::call_site());

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);
        if *is_top_level {
            return match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    let #variable_ident = #variable;
                    #render_fn(#render_ref_code #variable_ident, #area, #buffer);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    let #variable_ident = #variable;
                    #frame .#frame_render_fn(#render_ref_code #variable_ident, #frame.area());
                },
            };
        }
    }
    quote! {{ let #variable_ident = #variable; #variable_ident }}
}
