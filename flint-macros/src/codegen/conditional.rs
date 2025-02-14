use super::{generate_widget_code, WidgetHandlerOptions};
use crate::{
    arg::{Arg, ArgKind},
    widget::{Widget, WidgetKind},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident};

/// Handles the generation of a conditional widget that shows different content based on a condition.
/// A conditional widget is a convenience wrapper around a layout widget that shows different content
/// based on a condition. Currently, it only supports if-else.
///
/// # Arguments
///
/// * `condition` - The expression that determines which branch to show
/// * `if_child` - The widget to show when the condition is true
/// * `else_child` - Optional widget to show when the condition is false
/// * `options` - Configuration options for widget generation
///
/// # Returns
///
/// A TokenStream containing the generated widget code for both branches
pub fn handle_conditional_widget(
    condition: &Expr,
    if_child: &Box<Widget>,
    else_child: &Option<Box<Widget>>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    // We cannot actually have a runtime if-statement, which doesn't render
    // a widget at all, due to how ratatui layouts work. Instead, we use a
    // layout with a single child that is conditionally shown or hidden.
    let layout = Widget {
        // Construct the constraints argument to be passed to the layout
        args: vec![Arg {
            value: syn::parse2(quote! {
                if #condition {
                    [Constraint::Min(0)] // Minimum size of 0 (to show the element)
                } else {
                    [Constraint::Length(0)] // Maximum length of 0 (to hide the element)
                }
            })
            .expect("Failed to parse constraints expression"),
            kind: ArgKind::Named(Ident::new("constraints", proc_macro2::Span::call_site())),
        }],

        // We need this to be a Layout widget, since the Conditional widget is
        // a convenience wrapper around a Layout widget.
        kind: WidgetKind::Layout {
            name: Ident::new("Layout", proc_macro2::Span::call_site()),
            children: vec![*if_child.clone()],
        },
    };

    // Similar to the above case, we construct the constraints argument to be passed to the layout
    // except they're the exact opposite of the if case.
    let else_layout = else_child.as_ref().map(|else_child| Widget {
        args: vec![Arg {
            value: syn::parse2(quote! {
                if #condition {
                    [Constraint::Length(0)]
                } else {
                    [Constraint::Min(0)]
                }
            })
            .expect("Failed to parse constraints expression"),
            kind: ArgKind::Named(Ident::new("constraints", proc_macro2::Span::call_site())),
        }],
        kind: WidgetKind::Layout {
            name: Ident::new("Layout", proc_macro2::Span::call_site()),
            children: vec![(**else_child).clone()],
        },
    });

    // Render the if layout (and the else layout, if it is required)
    // by generating the code for the if layout and the else layout
    let if_layout = generate_widget_code(&layout, options);

    if let Some(else_layout) = else_layout {
        let else_code = generate_widget_code(&else_layout, options);
        quote! {
            {
                #if_layout
                #else_code
            }
        }
    } else {
        if_layout
    }
}
