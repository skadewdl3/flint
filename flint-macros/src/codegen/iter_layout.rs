/// Crate imports for widget handling functionality.
use crate::{
    arg::{Arg, ArgKind},
    codegen::util::generate_unique_id,
    widget::{Widget, WidgetKind, WidgetRenderer},
    MacroInput,
};

/// Super module imports for widget code generation.
use super::{generate_widget_code, WidgetHandlerOptions};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, Pat};

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
    widget: &Widget,
    loop_var: &Pat,
    iter: &Expr,
    child: &Box<Widget>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        parent_id,
        child_index,
        input,
    } = options;

    let args = &widget.args;
    let layout_index = generate_unique_id() as usize;
    let layout_ident =
        proc_macro2::Ident::new(&format!("layout_{}", layout_index), Span::call_site());
    let parent_ident = proc_macro2::Ident::new(&format!("chunks_{}", parent_id), Span::call_site());
    let iterator_index_ident = proc_macro2::Ident::new(
        &format!("iterator_index_{}", layout_index),
        Span::call_site(),
    );

    let positional_args: Vec<_> = args
        .iter()
        .filter_map(|arg| match &arg.kind {
            ArgKind::Positional => Some(&arg.value),
            _ => None,
        })
        .collect();

    let mut layout_code = quote! {
        let mut #layout_ident = ratatui::layout::Layout::default(#(#positional_args),*)
    };

    // Add named arguments as method calls
    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            layout_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    layout_code.extend(quote! { ; });

    let new_options = WidgetHandlerOptions::new(false, layout_index, *child_index, input);
    let child_widget = generate_widget_code(child, &new_options);

    match input {
        MacroInput::Ui { renderer, .. } => {
            // Create chunks vector
            let chunks_ident =
                proc_macro2::Ident::new(&format!("chunks_{}", layout_index), Span::call_site());

            // Split the area - for top level use frame.area(), for nested use the parent's chunk
            let split_code = if *is_top_level {
                match renderer {
                    WidgetRenderer::Area { area, .. } => {
                        quote! {
                            let #chunks_ident = #layout_ident.split(#area);
                        }
                    }

                    WidgetRenderer::Frame(frame) => {
                        quote! {
                            let #chunks_ident = #layout_ident.split(#frame .area());
                        }
                    }
                }
            } else {
                quote! {
                    let #chunks_ident = #layout_ident.split(#parent_ident[#child_index]);
                }
            };

            let mut render_statements = quote! {};
            match child.kind {
                // Layout widgets don't return an actual widget, so we don't call frame.render_widget on them
                // Instead, their children are rendered recursively
                WidgetKind::Layout { .. } | WidgetKind::IterLayout { .. } => {
                    render_statements.extend(quote! {
                        #child_widget
                    });
                }

                // For other widgets (Variable and Constructor), we call frame.render_widget on them
                // since they actually retturn something that implements ratatui::Widget
                _ => {
                    render_statements.extend(match renderer {
                        WidgetRenderer::Area { buffer, .. } => {
                            quote! {
                                for (#iterator_index_ident, #loop_var) in #iter.enumerate() {
                                    #child_widget.render(#chunks_ident[#iterator_index_ident], #buffer);
                                }
                            }
                        }

                        WidgetRenderer::Frame(frame) => {
                            quote! {
                                for (#iterator_index_ident, #loop_var) in #iter.enumerate() {
                                    #frame .render_widget(#child_widget, #chunks_ident[#iterator_index_ident]);
                                }
                            }
                        }
                    });
                }
            }

            quote! {
                {
                    #layout_code
                    #split_code
                    #render_statements
                }
            }
        }

        MacroInput::Raw { .. } => {
            let wrapper_code = quote! {

                use ratatui::{
                    buffer::Buffer,
                    layout::{Layout, Rect},
                    widgets::Widget,
                };

                pub struct IterLayoutWrapper<'a, I>
                where
                    I: Iterator
                {
                    layout: Layout,
                    iterator: I,
                    render_fn: Box<dyn Fn(I::Item, &Rect, &mut Buffer) + 'a>,
                }

                impl<'a, I> IterLayoutWrapper<'a, I>
                where
                    I: Iterator
                {
                    pub fn new<F>(
                        layout: Layout,
                        iterator: I,
                        render_fn: F
                    ) -> Self
                    where
                        F: Fn(I::Item, &Rect, &mut Buffer) + 'a
                    {
                        Self {
                            layout,
                            iterator,
                            render_fn: Box::new(render_fn),
                        }
                    }
                }

                impl<'a, I> Widget for IterLayoutWrapper<'a, I>
                where
                    I: Iterator
                {
                    fn render(self, area: Rect, buf: &mut Buffer) {
                        let chunks = self.layout.split(area);
                        for (chunk, item) in chunks.into_iter().zip(self.iterator) {
                            (self.render_fn)(item, chunk, buf);
                        }
                    }
                }
            };

            let render_statements = quote! {
                IterLayoutWrapper::new(
                    #layout_ident,
                    #iter,
                    |item, area, buf| {
                        let #loop_var = item;
                        let widget = #child_widget;
                        widget.render(*area, buf);
                    }
                )
            };

            return quote! {{
                #layout_code
                #wrapper_code
                #render_statements
            }};
        }
    }

    // if let MacroInput::Ui { renderer, widget } = input {
    // } else {
    // }
}
