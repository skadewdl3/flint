/// Crate imports for widget handling functionality.
use crate::{
    arg::{Arg, ArgKind},
    codegen::util::generate_unique_id,
    widget::{Widget, WidgetKind, WidgetRenderer},
    MacroInput,
};

/// Super module imports for widget code generation.
use super::{generate_widget_code, WidgetHandlerOptions};
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
    loop_var: &Pat,
    iter: &Expr,
    child: &Box<Widget>,
    options: &WidgetHandlerOptions,
) -> TokenStream {
    let WidgetHandlerOptions {
        is_top_level,
        input,
        child_index,
        ..
    } = options;

    match input {
        MacroInput::Ui { widget, renderer } => {
            if *is_top_level {
                let args = widget.args.clone();
                let layout = Widget {
                    // Construct the constraints argument to be passed to the layout
                    args,

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
                quote! {
                    {
                        #code
                    }
                }
            }
        }

        MacroInput::Raw { widget } => {
            if *is_top_level {
                let args = widget.args.clone();
                let layout_index = generate_unique_id() as usize;
                let layout_ident = proc_macro2::Ident::new(
                    &format!("layout_{}", layout_index),
                    proc_macro2::Span::call_site(),
                );
                let new_options =
                    WidgetHandlerOptions::new(false, layout_index, *child_index, input);
                let widget_code = generate_widget_code(child, &new_options);

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

                layout_code.extend(quote! {

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
                });

                layout_code.extend(quote! {
                    IterLayoutWrapper::new(
                        #layout_ident,
                        #iter,
                        |item, area, buf| {
                            let #loop_var = item;
                            let widget = #widget_code;
                            widget.render(*area, buf);
                        }
                    )
                });

                return quote! {{
                    #layout_code
                }};
            }
            panic!("You cannot use iterlayouts in the widget!() macro")
        }
    }

    // if let MacroInput::Ui { renderer, widget } = input {
    // } else {
    // }
}
