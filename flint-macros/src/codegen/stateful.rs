/// Required imports for handling stateful widgets
use crate::{
    codegen::{
        generate_widget_code, util::get_stateful_render_function, wrapper::get_stateful_wrapper,
    },
    widget::{Widget, WidgetRenderer},
    MacroInput,
};

use super::WidgetHandlerOptions;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Handles code generation for stateful widgets that maintain their own state as a separate entity.
/// This includes generating render functions and wrappers for widgets that need to track and update
/// internal state between renders. The handler deals with both reference and owned stateful widgets,
/// along with the complexities of rendering in Areas or Frames.
///
/// # Arguments
///
/// * `widget` - The widget definition containing render and state information
/// * `state` - The state expression for this stateful widget
/// * `child` - The child widget that this stateful widget wraps
/// * `options` - Configuration options for widget handling
///
/// # Returns
///
/// Returns a TokenStream containing the generated code for the stateful widget
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

    // Generate reference code based on whether widget should be passed by reference
    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    // Create new options for child widget code generation
    let new_options = WidgetHandlerOptions::new(false, *parent_id, *child_index, input);
    let child_widget = generate_widget_code(child, &new_options);

    if let MacroInput::Ui { renderer, .. } = input {
        let (stateful_render_fn, stateful_frame_render_fn) = get_stateful_render_function(widget);
        if *is_top_level {
            return match renderer {
                // Generate code for rendering in an Area
                WidgetRenderer::Area { area, buffer } => quote! {
                    #stateful_render_fn(#render_ref_code #child_widget, #area, #buffer, #state);
                },

                // Generate code for rendering in a Frame
                WidgetRenderer::Frame(frame) => quote! {
                    #frame .#stateful_frame_render_fn(#render_ref_code #child_widget, #frame.area(), #state);
                },
            };
        } else {
            child_widget
        }
    } else {
        // Handle stateful wrapper generation
        let stateful_wrapper = match child.render_ref {
            true => {
                panic!(
                    "The widget!() macro doesn't support rendering StatefulWidgetRef widgets yet."
                )
            }

            false => get_stateful_wrapper(),
        };

        // Generate initialization code for the stateful wrapper
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
