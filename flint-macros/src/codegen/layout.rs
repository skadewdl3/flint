use super::WidgetHandlerOptions;
use crate::{
    arg::ArgKind,
    codegen::{generate_widget_code, util::generate_unique_id},
    widget::{Widget, WidgetKind, WidgetRenderer},
    MacroInput,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Handles the code generation for a layout widget. This widget is responsible for
/// arranging its child widgets within a specified layout. It reduces the complexity of layout
/// management by automatically rendering the children in the correct area of the layout
/// according to their order.
///
/// # Arguments
///
/// * `widget` - The layout widget to handle
/// * `name` - The identifier for this layout
/// * `children` - Vector of child widgets contained in this layout
/// * `options` - Configuration options for widget handling
///
/// # Returns
///
/// A TokenStream containing the generated code for this layout widget and its children
pub fn handle_layout_widget(
    widget: &Widget,
    name: &Ident,
    children: &Vec<Widget>,
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
    let layout_ident = proc_macro2::Ident::new(&format!("layout_{}", layout_index), name.span());
    let parent_ident = proc_macro2::Ident::new(&format!("chunks_{}", parent_id), name.span());

    let positional_args: Vec<_> = args
        .iter()
        .filter_map(|arg| match &arg.kind {
            ArgKind::Positional => Some(&arg.value),
            _ => None,
        })
        .collect();

    let mut layout_code = quote! {
        let mut #layout_ident = #name::default(#(#positional_args),*)
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

    // Always end with semicolon after configuration
    //

    match input {
        MacroInput::Raw { .. } => {
            layout_code.extend(quote! {
                let mut children: Vec<Box<dyn Fn(Rect, &mut Buffer)>> = Vec::new();
            });

            layout_code.extend(quote! {


                        use ratatui::{
                            buffer::Buffer,
                            layout::{Layout, Rect},
                            widgets::Widget,
                        };

                        pub struct LayoutWrapper<'a> {
                            layout: Layout,
                            children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>,
                        }

                        impl<'a> LayoutWrapper<'a> {
                            pub fn new(layout: Layout, children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>) -> Self {
                                Self {
                                    layout,
                                    children,
                                }
                            }
                        }

                        impl<'a> Widget for LayoutWrapper<'a> {
                            fn render(self, area: Rect, buf: &mut Buffer) {
                                let chunks = self.layout.split(area);
                                for (idx, render_fn) in self.children.into_iter().enumerate() {
                                    render_fn(chunks[idx], buf);
                                }
                            }
                        }
                    });

            for (idx, child) in children.iter().enumerate() {
                let new_options = WidgetHandlerOptions::new(false, layout_index, idx, input);

                let child_widget = generate_widget_code(child, &new_options);

                layout_code.extend(quote! {
                    children.push(Box::new(|area, buf| {
                        #child_widget.render(area, buf);
                    }));
                });
            }

            layout_code.extend(quote! {
                LayoutWrapper::new(#layout_ident, children)
            });

            quote! {
                {
                    #layout_code
                }
            }
        }

        MacroInput::Ui { renderer, .. } => {
            // Create chunks vector
            let chunks_ident =
                proc_macro2::Ident::new(&format!("chunks_{}", layout_index), name.span());

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

            for (idx, child) in children.iter().enumerate() {
                let new_options = WidgetHandlerOptions::new(false, layout_index, idx, input);

                let child_widget = generate_widget_code(child, &new_options);

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
                                    #child_widget.render(#chunks_ident[#idx], #buffer)
                                }
                            }

                            WidgetRenderer::Frame(frame) => {
                                quote! {
                                    #frame .render_widget(#child_widget, #chunks_ident[#idx]);
                                }
                            }
                        });
                    }
                }
            }

            // Combine everything into a block
            quote! {
                {
                    #layout_code
                    #split_code
                    #render_statements
                }
            }
        }
    }
}
