use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};

#[derive(Debug)]
struct UiMacroInput {
    frame: Ident,
    widget: Widget,
}

#[derive(Debug, Clone)]
enum WidgetKind {
    Constructor(Ident, Ident),
    Layout(Ident, Vec<Widget>),
}

#[derive(Debug, Clone)]
struct Widget {
    kind: WidgetKind,
    args: Vec<Arg>,
}

#[derive(Debug, Clone)]
enum ArgKind {
    Positional,
    Named(Ident),
}

#[derive(Debug, Clone)]
struct Arg {
    value: Expr,
    kind: ArgKind,
}

impl Parse for UiMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let frame = input.parse()?;
        input.parse::<Token![=>]>()?;
        let widget = input.parse()?;

        Ok(UiMacroInput { frame, widget })
    }
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse widget name
        let widget_name = input.parse::<Ident>()?;

        let constructor_fn = if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            input.parse::<Ident>()?
        } else {
            Ident::new("default", widget_name.span())
        };

        // Parse arguments in parentheses
        let args = if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let args_punctuated = Punctuated::<Arg, Token![,]>::parse_terminated(&content)?;
            args_punctuated.into_iter().collect()
        } else {
            vec![]
        };

        // Check if this is a layout widget
        let mut kind = if is_layout_widget(&widget_name) {
            WidgetKind::Layout(widget_name, vec![])
        } else {
            WidgetKind::Constructor(widget_name, constructor_fn)
        };

        if let WidgetKind::Constructor(_, _) = kind {
            return Ok(Widget { kind, args });
        }

        // Parse child widgets in braces if present
        if input.peek(token::Brace) {
            let content;
            syn::braced!(content in input);

            if let WidgetKind::Layout(_, ref mut children) = kind {
                // Parse children as a punctuated sequence
                let child_widgets = Punctuated::<Widget, Token![,]>::parse_terminated(&content)?;
                children.extend(child_widgets);
            } else {
                return Err(input.error("Only Layout widgets can have child elements"));
            }
        }

        Ok(Widget { kind, args })
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check if we have a named parameter (identified by an identifier followed by a colon)
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident) && input.peek2(Token![:]) {
            // Parse named parameter
            let name = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let value = input.parse::<Expr>()?;

            Ok(Arg {
                value,
                kind: ArgKind::Named(name),
            })
        } else {
            // Parse positional parameter
            let value = input.parse::<Expr>()?;

            Ok(Arg {
                value,
                kind: ArgKind::Positional,
            })
        }
    }
}

fn is_layout_widget(name: &Ident) -> bool {
    name.to_string() == "Layout"
}

fn generate_widget_code(
    widget: &Widget,
    is_top_level: bool,
    parent_index: usize,
    child_index: usize,
    frame: &Ident,
) -> proc_macro2::TokenStream {
    match &widget.kind {
        WidgetKind::Constructor(name, constructor) => {
            let args = &widget.args;

            let positional_args: Vec<_> = args
                .iter()
                .filter_map(|arg| match &arg.kind {
                    ArgKind::Positional => Some(&arg.value),
                    _ => None,
                })
                .collect();

            // Start with constructor call including all positional arguments
            let mut widget = quote! {
                #name :: #constructor(#(#positional_args),*)
            };

            for arg in args {
                if let ArgKind::Named(name) = &arg.kind {
                    let value = &arg.value;
                    widget.extend(quote! {
                        .#name(#value)
                    });
                }
            }

            if is_top_level {
                quote! {
                    frame.render_widget(#widget, frame.area());
                }
            } else {
                widget
            }
        }
        WidgetKind::Layout(name, children) => {
            let args = &widget.args;
            let layout_index = generate_unique_id() as usize;
            let layout_ident =
                proc_macro2::Ident::new(&format!("layout_{}", layout_index), name.span());
            let parent_ident =
                proc_macro2::Ident::new(&format!("chunks_{}", parent_index), name.span());

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

            // Always end with semicolon after configuration
            layout_code.extend(quote! { ; });

            // Create chunks vector
            let chunks_ident =
                proc_macro2::Ident::new(&format!("chunks_{}", layout_index), name.span());

            // Split the area - for top level use frame.area(), for nested use the parent's chunk
            let split_code = if is_top_level {
                quote! {
                    let #chunks_ident = #layout_ident.split(#frame .area());
                }
            } else {
                quote! {
                    let #chunks_ident = #layout_ident.split(#parent_ident[#child_index]);
                }
            };

            let mut render_statements = quote! {};
            for (idx, child) in children.iter().enumerate() {
                let child_widget = generate_widget_code(child, false, layout_index, idx, frame);

                if let WidgetKind::Layout(_, _) = child.kind {
                    render_statements.extend(quote! {
                        #child_widget
                    });
                } else {
                    render_statements.extend(quote! {
                        #frame .render_widget(#child_widget, #chunks_ident[#idx]);
                    });
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

fn get_constructor_name(name: Ident) -> proc_macro2::TokenStream {
    match name.to_string().as_str() {
        "Text" => quote! { ::raw },
        _ => quote! { ::default },
    }
}

fn generate_unique_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let UiMacroInput { frame, widget, .. } = parse_macro_input!(input as UiMacroInput);
    let output = generate_widget_code(&widget, true, 0, 0, &frame);
    output.into()
}
