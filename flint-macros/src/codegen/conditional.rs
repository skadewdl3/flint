use super::{util::get_render_function, WidgetHandlerOptions};
use crate::{
    codegen::{generate_widget_code, wrapper::get_conditional_wrapper},
    widget::{Widget, WidgetRenderer},
    MacroInput,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Handles the rendering of a conditional widget based on a boolean expression.
///
/// This function allows for conditional rendering of widgets using the `if-else` syntax.
/// It takes a condition and widgets to render for both the `if` and optional `else` cases.
///
/// # Arguments
///
/// * `widget` - The parent conditional widget containing the condition, and if/else child widgets information
/// * `condition` - The boolean expression that determines which branch to render
/// * `if_child` - The widget to render when the condition is true
/// * `else_child` - Optional widget to render when the condition is false
/// * `options` - Configuration options for widget handling, including top level status
///
/// # Returns
///
/// A `TokenStream` containing the generated code to conditionally render widgets:
/// - For top-level widgets: Direct render function calls in an if-else block
/// - For nested widgets: A `ConditionalWrapper` initialization
pub fn handle_conditional_widget(
    widget: &Widget,
    condition: &Expr,
    if_child: &Box<Widget>,
    else_child: &Option<Box<Widget>>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    // Extract relevant options
    let WidgetHandlerOptions {
        is_top_level,
        input,
        parent_id,
        child_index,
        ..
    } = options;

    // Determine if we need to render by reference
    let render_ref_code = match widget.render_ref {
        true => quote! {&},
        false => quote! {},
    };

    // Create new options for child widgets (not top level)
    let new_options = WidgetHandlerOptions::new(false, *parent_id, *child_index, input);
    // Generate code for the if branch widget
    let if_child_widget = generate_widget_code(if_child, &new_options);

    // Get the conditional wrapper type for nested widgets
    let conditional_wrapper = get_conditional_wrapper();

    // Handle UI rendering case
    if let MacroInput::Ui { renderer, .. } = input {
        let (render_fn, frame_render_fn) = get_render_function(widget);

        // Generate code for else branch if it exists
        let else_child_widget = match else_child {
            Some(else_child) => {
                let else_child_widget = generate_widget_code(else_child, &new_options);
                else_child_widget
            }
            None => quote! {},
        };

        // For top level widgets, generate direct conditional render calls
        if *is_top_level {
            return match renderer {
                // Render to an area with buffer
                WidgetRenderer::Area { area, buffer } => quote! {
                    if #condition {
                        #render_fn(#render_ref_code #if_child_widget, #area, #buffer);
                    } else {
                        #render_fn(#render_ref_code #else_child_widget, #area, #buffer);
                    }
                },

                // Render to a frame
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

    // For nested widgets, wrap in a ConditionalWrapper
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
