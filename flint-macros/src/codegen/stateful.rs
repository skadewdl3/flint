use crate::{
    codegen::{generate_widget_code, util::get_stateful_render_function},
    widget::{Widget, WidgetRenderer},
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
pub fn handle_stateful_widget(
    widget: &Widget,
    state: &Expr,
    child: &Box<Widget>,
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
    let child_widget = generate_widget_code(child, &new_options);

    if let MacroInput::Ui { renderer, .. } = input {
        let (stateful_render_fn, stateful_frame_render_fn) = get_stateful_render_function(widget);
        if *is_top_level {
            return match renderer {
                // TODO: if widget is stateful, pass in the state
                WidgetRenderer::Area { area, buffer } => quote! {
                    #stateful_render_fn(#render_ref_code #child_widget, #area, #buffer, #state);
                },

                WidgetRenderer::Frame(frame) => quote! {
                    #frame .#stateful_frame_render_fn(#render_ref_code #child_widget, #frame.area(), #state);
                },
            };
        } else {
            child_widget
        }
    } else {
        let stateful_wrapper = match child.render_ref {
            true => {
                panic!(
                    "The widget!() macro doesn't support rendering StatefulWidgetRef widgets yet."
                )
            }

            false => quote! {
                use std::cell::RefCell;
                use ratatui::{
                    widgets::{StatefulWidget, Widget},
                    layout::Rect,
                    buffer::Buffer,
                };

                pub struct StatefulWrapper<'a, W, S>
                where
                    W: StatefulWidget<State = S>,
                {
                    widget: W,
                    state: RefCell<&'a mut S>,
                }

                impl<'a, W, S> StatefulWrapper<'a, W, S>
                where
                    W: StatefulWidget<State = S>,
                {
                    /// Creates a new StatefulWrapper with the given widget and state
                    pub fn new(widget: W, state: &'a mut S) -> Self {
                        Self {
                            widget,
                            state: RefCell::new(state)
                        }
                    }
                }

                impl<'a, W, S> Widget for StatefulWrapper<'a, W, S>
                where
                    W: StatefulWidget<State = S>,
                {
                    fn render(self, area: Rect, buf: &mut Buffer) {
                        let mut state = self.state.borrow_mut();
                        ratatui::widgets::StatefulWidget::render(self.widget, area, buf, &mut *state);
                    }
                }
            },
        };

        let stateful_wrapper_init = match child.render_ref {
            true => quote! {
                StatefulRefWrapper::new(#render_ref_code #child_widget, #state)
            },
            false => quote! {
                StatefulWrapper::new(#child_widget, #state)
            },
        };
        quote! {
            {
                #stateful_wrapper
                #stateful_wrapper_init
            }
        }
    }
}
