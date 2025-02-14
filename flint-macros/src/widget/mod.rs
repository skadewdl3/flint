mod util;

use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};
use util::is_layout_widget;

use crate::arg::Arg;

#[derive(Debug, Clone)]
pub enum WidgetKind {
    Constructor {
        name: Ident,
        constructor: Ident,
    },
    Layout {
        name: Ident,
        children: Vec<Widget>,
    },
    Variable {
        expr: Expr,
    },
    Conditional {
        condition: Expr,
        if_child: Box<Widget>,
        else_child: Option<Box<Widget>>,
    },
}

#[derive(Debug, Clone)]
pub struct Widget {
    pub kind: WidgetKind,
    pub args: Vec<Arg>,
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(token::Brace) {
            let content;
            syn::braced!(content in input);

            // Check if the content starts with another left brace
            if content.peek(token::Brace) {
                let inner_content;
                syn::braced!(inner_content in content);
                let expr: Expr = inner_content.parse()?;

                return Ok(Widget {
                    kind: WidgetKind::Variable { expr },
                    args: vec![],
                });
            }
        }

        // Parse widget name
        let widget_name = input.parse::<Ident>()?;

        if widget_name == "If" {
            let content;
            syn::parenthesized!(content in input);
            let condition = content.parse::<Expr>()?;

            let brace_content;
            braced!(brace_content in input);
            let child = brace_content.parse::<Widget>()?;

            // Check for Else keyword
            let else_child = if input.peek(Ident) {
                let else_kw: Ident = input.parse()?;
                if else_kw == "Else" {
                    let else_content;
                    braced!(else_content in input);
                    Some(Box::new(else_content.parse::<Widget>()?))
                } else {
                    return Err(input.error("Expected 'Else' keyword"));
                }
            } else {
                None
            };

            return Ok(Widget {
                kind: WidgetKind::Conditional {
                    condition,
                    if_child: Box::new(child),
                    else_child,
                },
                args: vec![],
            });
        }

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
            WidgetKind::Layout {
                name: widget_name,
                children: vec![],
            }
        } else {
            WidgetKind::Constructor {
                name: widget_name,
                constructor: constructor_fn,
            }
        };

        if let WidgetKind::Constructor { .. } = kind {
            return Ok(Widget { kind, args });
        }

        // Parse child widgets in braces if present
        if input.peek(token::Brace) {
            let content;
            syn::braced!(content in input);

            if let WidgetKind::Layout {
                ref mut children, ..
            } = kind
            {
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
