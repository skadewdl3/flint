use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use ratatui::text::Text;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};

#[derive(Debug, Clone)]
enum WidgetKind {
    Constructor(Ident),
    Variable(Ident),
    Layout(Ident, Vec<Widget>),
}

#[derive(Debug, Clone)]
struct Widget {
    kind: WidgetKind,
    args: Vec<Arg>,
}

#[derive(Debug, Clone)]
struct Arg {
    name: Ident,
    value: Expr,
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse variable based widget
        if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let var_name = content.parse::<Ident>()?;
            return Ok(Widget {
                kind: WidgetKind::Variable(var_name),
                args: vec![],
            });
        }

        // Parse constructor based widget
        // Parse opening angle bracket
        input.parse::<Token![<]>()?;

        // Parse widget name/type
        let widget_name = input.parse::<Ident>()?;
        let kind = if is_layout_widget(&widget_name) {
            WidgetKind::Layout(widget_name, vec![])
        } else {
            WidgetKind::Constructor(widget_name)
        };

        // Parse arguments
        let mut args = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let name = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            // Require curly braces around the value
            let content;
            braced!(content in input);
            let value = content.parse::<Expr>()?;

            args.push(Arg { name, value });
        }

        let mut children = Vec::new();

        // Handle self-closing tags - only allow for non-layout widgets
        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            if matches!(kind, WidgetKind::Layout(_, _)) {
                return Err(input.error("Layout widgets cannot be self closing"));
            }

            return Ok(Widget { kind, args });
        }

        // Parse regular closing bracket
        input.parse::<Token![>]>()?;

        // Parse children until we hit the closing tag
        while !input.is_empty() && !input.peek(Token![<]) {
            if let Ok(child) = input.parse::<Widget>() {
                children.push(child);
            }
        }

        // Parse closing tag
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let closing_name = input.parse::<Ident>()?;

        // Verify matching tags
        match &kind {
            WidgetKind::Constructor(name) | WidgetKind::Layout(name, _) => {
                if name != &closing_name {
                    return Err(input.error("Mismatched opening and closing tags"));
                }
            }
            _ => {}
        }

        input.parse::<Token![>]>()?;

        match kind {
            WidgetKind::Layout(name, _) => Ok(Widget {
                kind: WidgetKind::Layout(name, children),
                args,
            }),
            _ => Ok(Widget { kind, args }),
        }
    }
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);

    // Convert the parsed Widget into actual Rust code
    let output = generate_widget_code(&widget);

    output.into()
}

// Helper function to determine if a widget is a layout widget
fn is_layout_widget(name: &Ident) -> bool {
    let name_str = name.to_string();
    matches!(name_str.as_str(), "Layout") // Add other layout widget names here if needed
}

fn get_constructor_name(name: &Ident) -> proc_macro2::TokenStream {
    match name {
        Text => quote! { ::raw },
        _ => quote! { ::default },
    }
}

fn get_constructor_arg(args: &Vec<Arg>) -> Option<(usize, &Arg)> {
    args.iter().enumerate().find(|(_, arg)| arg.name == "cons")
}

fn generate_widget_code(widget: &Widget) -> proc_macro2::TokenStream {
    match &widget.kind {
        WidgetKind::Layout(name, children) => {
            quote! {}
        }
        WidgetKind::Constructor(name) => {
            let args = &widget.args;

            // let children = widget.children.iter().map(generate_widget_code);
            let constructor_name = get_constructor_name(name);
            let constructor_arg = get_constructor_arg(args);

            let mut widget = match constructor_arg {
                Some((_, Arg { value, .. })) => {
                    quote! {
                        #name #constructor_name(#value)
                    }
                }
                None => {
                    quote! {
                        #name #constructor_name()
                    }
                }
            };

            for (i, arg) in args.iter().enumerate() {
                if let Some((index, _)) = constructor_arg {
                    if i == index {
                        continue;
                    }
                }

                let name = &arg.name;
                let value = &arg.value;

                widget.extend(quote! {
                    .#name(#value)
                });
            }

            widget
        }
        WidgetKind::Variable(name) => quote! { #name },
    }
}
