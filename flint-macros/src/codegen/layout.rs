use super::WidgetHandlerOptions;
use crate::{
    arg::ArgKind,
    codegen::{
        generate_widget_code,
        util::{generate_unique_id, get_render_function, get_stateful_render_function},
        wrapper::get_layout_wrapper,
    },
    widget::{Widget, WidgetKind, WidgetRenderer},
    MacroInput,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// This module handles code generation for layout widgets in a UI system. Layout widgets are containers
/// that organize and arrange their child widgets according to specified layout rules.
///
/// The code generation process involves:
/// 1. Creating a new layout instance with provided arguments
/// 2. Configuring the layout with any named parameters
/// 3. Handling child widgets differently based on the input type (Raw vs UI)
/// 4. For Raw input: Creating a vector of render functions for children
/// 5. For UI input: Splitting the available area into chunks and rendering children into those chunks
///
/// The function supports different types of child widgets:
/// - Layout widgets: Rendered recursively
/// - Constructor widgets: Always rendered statelessly
/// - Stateful widgets: Rendered with state management
/// - Other widgets: Rendered normally into layout chunks
///
/// # Arguments
///
/// * `widget` - The layout widget configuration to process
/// * `name` - The identifier for this layout instance
/// * `children` - Vector of child widgets to be arranged in this layout
/// * `options` - Configuration options including parent/child relationships and render mode
///
/// # Returns
///
/// A TokenStream containing the complete generated code for this layout and its children
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

    // Extract positional arguments from widget configuration
    let positional_args: Vec<_> = args
        .iter()
        .filter_map(|arg| match &arg.kind {
            ArgKind::Positional => Some(&arg.value),
            _ => None,
        })
        .collect();

    // Begin constructing layout initialization code
    let mut layout_code = quote! {
        let mut #layout_ident = #name::default(#(#positional_args),*)
    };

    // Configure layout with named arguments
    for arg in args {
        if let ArgKind::Named(name) = &arg.kind {
            let value = &arg.value;
            layout_code.extend(quote! {
                .#name(#value)
            });
        }
    }

    layout_code.extend(quote! { ; });

    match input {
        // Raw mode: Create vector of render functions
        MacroInput::Raw { .. } => {
            layout_code.extend(quote! {
                let mut children: Vec<Box<dyn Fn(Rect, &mut Buffer)>> = Vec::new();
            });

            layout_code.extend(get_layout_wrapper());

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

        // UI mode: Split area into chunks and render children
        MacroInput::Ui { renderer, .. } => {
            let chunks_ident =
                proc_macro2::Ident::new(&format!("chunks_{}", layout_index), name.span());

            // Generate area splitting code based on layout level
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

            // Process each child widget
            for (idx, child) in children.iter().enumerate() {
                let new_options = WidgetHandlerOptions::new(false, layout_index, idx, input);
                let (render_fn, frame_render_fn) = get_render_function(child);
                let (stateful_render_fn, stateful_frame_render_fn) =
                    get_stateful_render_function(child);
                let child_widget = generate_widget_code(child, &new_options);
                let render_ref_code = match child.render_ref {
                    true => quote! {&},
                    false => quote! {},
                };

                match child.kind {
                    // Layout widgets render recursively
                    WidgetKind::Layout { .. } | WidgetKind::IterLayout { .. } => {
                        render_statements.extend(quote! {
                            #child_widget
                        });
                    }

                    // Constructor widgets render statelessly
                    WidgetKind::Constructor { .. } => {
                        let (render_fn, frame_render_fn) = get_render_function(widget);
                        render_statements.extend(match renderer {
                            WidgetRenderer::Area { buffer, .. } =>  {
                                quote! {
                                    #render_fn(#render_ref_code #child_widget, #chunks_ident[#idx], #buffer);
                                }
                            }

                            WidgetRenderer::Frame(frame) => quote! {
                                #frame .#frame_render_fn(#render_ref_code #child_widget, #chunks_ident[#idx]);
                            },
                        });
                    }

                    // Stateful widgets include state in rendering
                    WidgetKind::Stateful { ref state, .. } => {
                        render_statements.extend(match renderer {
                            WidgetRenderer::Area {  buffer, .. } => quote! {
                                #stateful_render_fn(#render_ref_code #child_widget, #chunks_ident[#idx], #buffer, #state);
                            },

                            WidgetRenderer::Frame(frame) => quote! {
                                #frame .#stateful_frame_render_fn(#render_ref_code #child_widget, #chunks_ident[#idx], #state);
                            },
                        });
                    }

                    // Standard widgets render normally
                    _ => {
                        render_statements.extend(match renderer {
                            WidgetRenderer::Area {  buffer, .. } => quote! {
                                #render_fn(#render_ref_code #child_widget, #chunks_ident[#idx], #buffer);
                            },

                            WidgetRenderer::Frame(frame) => quote! {
                                #frame .#frame_render_fn(#render_ref_code #child_widget, #chunks_ident[#idx]);
                            },
                        });
                    }
                }
            }

            // Combine all generated code
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
