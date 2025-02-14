mod util;

use crate::arg::Arg;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};
use util::is_layout_widget;

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
    /// A conditional widget with if/else branches
    Conditional {
        /// The condition expression
        condition: Expr,
        /// The widget to show if condition is true
        if_child: Box<Widget>,
        /// Optional widget to show if condition is false
        else_child: Option<Box<Widget>>,
    },
}

/// Represents a widget with its kind and arguments
#[derive(Debug, Clone)]
pub struct Widget {
    /// The kind of widget
    pub kind: WidgetKind,
    /// Arguments passed to the widget
    pub args: Vec<Arg>,
}

/// Parser implementation for Widget
impl Parse for Widget {
    /// Parses a widget from a token stream
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

        // If this widget name is "If", it's a conditional widget
        if widget_name == "If" {
            // Parse the condition (which evaluates to a boolean) given in
            // the parantheses
            let content;
            syn::parenthesized!(content in input);
            let condition = content.parse::<Expr>()?;

            // The content in braces is rendered if the condition is true
            // The braces can contain only one single widget. So if multiple child elements
            // are required, they must be nested in a Layout widget.
            let brace_content;
            braced!(brace_content in input);
            let child = brace_content.parse::<Widget>()?;

            // Optionally, an Else clause may follow the If clause
            let else_child = if input.peek(Ident) {
                let else_kw: Ident = input.parse()?;
                if else_kw == "Else" {
                    // If it exists, we extract another single widget from the braces
                    // which will be rendered if the condition provided to the If clause is false.
                    let else_content;
                    braced!(else_content in input);
                    Some(Box::new(else_content.parse::<Widget>()?))
                } else {
                    return Err(input.error("Expected 'Else' keyword"));
                }
            } else {
                None
            };

            // If this was a conditional widget, we're done, since we've
            // extracted the condition and children for both branches.
            return Ok(Widget {
                kind: WidgetKind::Conditional {
                    condition,
                    if_child: Box::new(child),
                    else_child,
                },
                args: vec![],
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

        // If this is a layout widget, we'll need to parse child widgets in braces
        // so create a field for that. No widgets except Layout widgets can have children,
        // that's why no other widget needs to parse child widgets.
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

        // If this is a constructor widget, we're done since we don't need to parse child widgets
        if let WidgetKind::Constructor { .. } = kind {
            return Ok(Widget { kind, args });
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

        Ok(Widget { kind, args })
    }
}
