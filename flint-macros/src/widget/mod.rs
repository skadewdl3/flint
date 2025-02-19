use crate::arg::{Arg, ArgKind};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Pat, Result, Token,
};

/// Represents the different kinds of widgets that can be parsed
#[derive(Debug, Clone)]
pub enum WidgetKind {
    /// A widget constructed directly using a constructor function
    Constructor {
        /// The name of the widget
        name: Ident,
        /// The constructor function name
        constructor: Ident,
    },
    /// A layout widget that can contain child widgets
    Layout {
        /// The name of the layout widget
        name: Ident,
        /// The child widgets contained in this layout
        children: Vec<Widget>,
    },
    /// A widget represented by a variable expression
    Variable {
        /// The expression representing the widget
        expr: Expr,
    },
    IterLayout {
        loop_var: Pat,
        iter: Expr,
        /// The child widgets contained in this layout
        child: Box<Widget>,
    },
}

#[derive(Debug)]
pub enum WidgetRenderer {
    Area { area: Expr, buffer: Expr },
    Frame(Expr),
}

impl Parse for WidgetRenderer {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let area = content.parse::<Expr>()?;
            content.parse::<Token![,]>()?;
            let buffer = content.parse::<Expr>()?;

            Ok(WidgetRenderer::Area { area, buffer })
        } else {
            let ident = input.parse()?;
            Ok(WidgetRenderer::Frame(ident))
        }
    }
}

/// Represents a widget with its kind and arguments
#[derive(Debug, Clone)]
pub struct Widget {
    /// The kind of widget
    pub kind: WidgetKind,
    /// Arguments passed to the widget
    pub args: Vec<Arg>,
    /// Whether this widget should be rendered as a reference
    pub render_ref: bool,
    /// Whether this widget maintains internal state
    pub stateful: bool,
}

/// Parser implementation for Widget
impl Parse for Widget {
    /// Parses a widget from a token stream
    fn parse(input: ParseStream) -> Result<Self> {
        // If we find an "&", this widget should be rendered as a reference
        let render_ref = if input.peek(Token![&]) {
            _ = input.parse::<Token![&]>().unwrap();
            true
        } else {
            false
        };

        // If we find a "{", then try to parse for a variable widget
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
                    render_ref,
                    stateful: false,
                });
            }
        }

        // Parse widget name
        let widget_name = input.parse::<Ident>()?;

        if widget_name == "For" {
            // Parse the condition (which evaluates to a boolean) given in
            // the parantheses
            let content;
            syn::parenthesized!(content in input);
            let loop_var = Pat::parse_multi_with_leading_vert(&content)?;
            content.parse::<Token![in]>()?;
            let iter = content.parse::<Expr>()?;

            // Parse named argument if it exists (separated by comma)
            let mut args = Vec::new();
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
                let temp = Punctuated::<Arg, Token![,]>::parse_terminated(&content)?;
                args = temp.into_iter().collect();
                if args.is_empty() {
                    return Err(input.error("For widgets must have at least 1 argument"));
                }
            }

            // The content in braces is rendered if the condition is true
            // The braces can contain only one single widget. So if multiple child elements
            // are required, they must be nested in a Layout widget.
            let brace_content;
            braced!(brace_content in input);

            let child = brace_content.parse::<Widget>()?;

            // If this was a conditional widget, we're done, since we've
            // extracted the condition and children for both branches.
            return Ok(Widget {
                kind: WidgetKind::IterLayout {
                    loop_var,
                    iter,
                    child: Box::new(child),
                },
                render_ref: false,
                stateful: false,
                args,
            });
        }

        // If the user provided a constructor function (like MyWidget::new)
        // use that function to create the widget, otherwise, use the
        // default constructor (MyWidget::default)
        let constructor_fn = if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            input.parse::<Ident>()?
        } else {
            Ident::new("default", widget_name.span())
        };

        // Parse positional and named arguments provided in parantheses.
        // Positional arguments are passed directly to the constructor function.
        // Named arguments are chained as function calls to the value
        // returned by the constructor function.
        let args = if input.peek(token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let args_punctuated = Punctuated::<Arg, Token![,]>::parse_terminated(&content)?;
            args_punctuated.into_iter().collect()
        } else {
            vec![]
        };

        let stateful = args
            .iter()
            .filter_map(|arg| {
                if let ArgKind::Named(ref ident) = arg.kind {
                    Some(ident.to_string())
                } else {
                    None
                }
            })
            .any(|name| name == "state");

        // If this is a layout widget, we'll need to parse child widgets in braces
        // so create a field for that. No widgets except Layout widgets can have children,
        // that's why no other widget needs to parse child widgets.
        let mut kind = if widget_name == "Layout" {
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

        // If this is a constructor widget, we're done since we don't need to parse child widgets
        if let WidgetKind::Constructor { .. } = kind {
            return Ok(Widget {
                kind,
                args,
                render_ref,
                stateful,
            });
        }

        // Since this is a layout widget, we'll need to parse child widgets in braces
        if input.peek(token::Brace) {
            let content;
            syn::braced!(content in input);

            if let WidgetKind::Layout {
                ref mut children, ..
            } = kind
            {
                // Parse the child widgets. Every child widget must be separated by a comma.
                // Child widgets can be any kind of widget, including other layout widgets.
                let child_widgets = Punctuated::<Widget, Token![,]>::parse_terminated(&content)?;
                children.extend(child_widgets);
            } else {
                return Err(input.error("Only Layout widgets can have child elements"));
            }
        }

        Ok(Widget {
            kind,
            args,
            render_ref: false,
            stateful: false,
        })
    }
}
