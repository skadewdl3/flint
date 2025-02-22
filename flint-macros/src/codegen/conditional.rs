use crate::{
    codegen::{generate_widget_code, util::get_stateful_render_function},
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
pub fn handle_conditional_widget(
    widget: &Widget,
    condition: &Expr,
    if_child: &Box<Widget>,
    else_child: &Option<Box<Widget>>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        input,
        parent_id,
        child_index,
        ..
    } = options;

    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    let new_options = WidgetHandlerOptions::new(false, *parent_id, *child_index, input);
    let if_child_widget = generate_widget_code(if_child, &new_options);

    let conditional_wrapper = quote! {
        use ratatui::{
            widgets::Widget,
            layout::Rect,
            buffer::Buffer,
        };

        pub struct ConditionalWrapper<W, E>
        where
            W: Widget,
            E: Widget,
        {
            if_widget: W,
            else_widget: Option<E>,
            condition: bool,
        }

        impl<W, E> ConditionalWrapper<W, E>
        where
            W: Widget,
            E: Widget,
        {
            /// Creates a new ConditionalWrapper with the given widget and condition
            pub fn new(if_widget: W, condition: bool) -> Self {
                Self {
                    if_widget,
                    else_widget: None,
                    condition,
                }
            }

            /// Adds an else widget to the conditional wrapper
            pub fn with_else(if_widget: W, else_widget: E, condition: bool) -> Self {
                Self {
                    if_widget,
                    else_widget: Some(else_widget),
                    condition,
                }
            }
        }

        impl<W, E> Widget for ConditionalWrapper<W, E>
        where
            W: Widget,
            E: Widget,
        {
            fn render(self, area: Rect, buf: &mut Buffer) {
                if self.condition {
                    self.if_widget.render(area, buf);
                } else if let Some(else_widget) = self.else_widget {
                    else_widget.render(area, buf);
                }
            }
        }
    };

    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);

        let else_child_widget = match else_child {
            Some(else_child) => {
                let else_child_widget = generate_widget_code(else_child, &new_options);
                else_child_widget
            }
            None => quote! {},
        };

        if *is_top_level {
            return match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    if #condition {
                        #render_fn(#render_ref_code #if_child_widget, #area, #buffer);
                    } else {
                        #render_fn(#render_ref_code #else_child_widget, #area, #buffer);
                    }
                },

                WidgetRenderer::Frame(frame) => quote! {
                    if #condition {
                        #frame .#frame_render_fn(#render_ref_code #if_child_widget, #frame.area());
                    } else {
                        #frame .#frame_render_fn(#render_ref_code #else_child_widget, #frame.area());
                    }
                },
            };
        }
    }

    let conditional_code = match else_child {
        Some(else_child) => {
            let else_child_widget = generate_widget_code(else_child, &new_options);
            quote! { ConditionalWrapper::with_else(#if_child_widget, #else_child_widget, #condition) }
        }
        None => quote! { ConditionalWrapper::new(#if_child_widget, #condition) },
    };

    quote! {
        {
            #conditional_wrapper
            #conditional_code
        }
    }

    // let stateful_wrapper_init = match child.render_ref {
    //             true => quote! {
    //                 StatefulRefWrapper::new(#render_ref_code #child_widget, #state)
    //             },
    //             false => quote! {
    //                 StatefulWrapper::new(#child_widget, #state)
    //             },
    //         };
}
