/// Crate imports for widget handling functionality.
use crate::{
    arg::{Arg, ArgKind},
    widget::{Widget, WidgetKind, WidgetRenderer},
    MacroInput,
};

/// Super module imports for widget code generation.
use super::{generate_widget_code, WidgetHandlerOptions};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident};

/// Handles generating code for an iteration-based layout widget.
///
/// # Arguments
///
/// * `loop_var` - The loop variable expression
/// * `iter` - The iterator expression
/// * `child` - The child widget to render in the loop
/// * `options` - Configuration options for widget handling
///
/// # Returns
///
/// A TokenStream containing the generated widget code
pub fn handle_iter_layout_widget(
    _loop_var: &Expr,
    iter: &Expr,
    child: &Box<Widget>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        input,
        ..
    } = options;

    if let MacroInput::Ui { renderer, widget } = input {
        if *is_top_level {
            let layout = Widget {
                // Construct the constraints argument to be passed to the layout
                args: vec![Arg {
                    value: syn::parse2(quote! {
                        vec![Constraint::Fill(1); #iter.clone().len()] // Maximum length of 0 (to hide the element)
                    })
                    .expect("Failed to parse constraints expression"),
                    kind: ArgKind::Named(Ident::new("constraints", proc_macro2::Span::call_site())),
                }],

                // We need this to be a Layout widget, since the Conditional widget is
                // a convenience wrapper around a Layout widget.
                kind: WidgetKind::Layout {
                    name: Ident::new("Layout", proc_macro2::Span::call_site()),
                    children: vec![widget.clone()],
                },
            };

            let widget_code = generate_widget_code(&layout, options);
            match renderer {
                WidgetRenderer::Area { .. } | WidgetRenderer::Frame(_) => {
                    quote! {
                        {
                            #widget_code
                        }
                    }
                }
            }
        } else {
            let code = generate_widget_code(child, options);
            match renderer {
                WidgetRenderer::Area { .. } => {
                    quote! {
                        {
                            #code
                        }
                    }
                }

                WidgetRenderer::Frame(_) => {
                    quote! {
                        {
                            #code
                        }
                    }
                }
            }
        }
    } else {
        panic!("You cannot use iterlayouts in the widget!() macro")
    }
}
