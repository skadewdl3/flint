use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    Expr, Ident, Result, Token,
};

/// Represents the kind of argument that can be passed
/// to a function or macro invocation.
#[derive(Debug, Clone)]
pub enum ArgKind {
    /// A positional argument with no named identifier
    Positional,
    /// A named argument with an identifier and value
    Named(Ident),
}

/// Represents a single argument in a function or macro invocation,
/// containing both the value and information about how it is passed
/// (positional or named).
#[derive(Debug, Clone)]
pub struct Arg {
    /// The actual value/expression of the argument
    pub value: Expr,
    /// Whether this is a positional or named argument
    pub kind: ArgKind,
}

impl Parse for Arg {
    /// Parses a single argument from a token stream.
    ///
    /// This will parse either:
    /// - A named argument in the form `name: value`
    /// - A shorthand named argument which is just an identifier (treated as `ident: ident`)
    /// - A positional argument which is any other expression
    ///
    /// # Examples:
    /// ```rust
    /// Widget("arg1", name: "value", shorthand, some_func())
    /// ```
    ///
    /// # Returns
    /// - `Ok(Arg)` if parsing succeeds
    /// - `Err(Error)` if parsing fails
    fn parse(input: ParseStream) -> Result<Self> {
        // Check for named parameter (identified by an identifier followed by a colon)
        if input.peek(Ident) && input.peek2(Token![:]) {
            let name = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let value = input.parse::<Expr>()?;

            return Ok(Arg {
                value,
                kind: ArgKind::Named(name),
            });
        }

        // Try parsing as an expression
        let fork = input.fork();
        let expr_result = fork.parse::<Expr>();

        match expr_result {
            Ok(expr) => {
                // If it parsed successfully as an expression, check if it's just an identifier
                if let Expr::Path(expr_path) = &expr {
                    if expr_path.path.segments.len() == 1
                        && !expr_path.path.segments[0].arguments.is_empty()
                    {
                        // If it's a path with arguments (like a function call), treat as positional
                        input.parse::<Expr>().map(|value| Arg {
                            value,
                            kind: ArgKind::Positional,
                        })
                    } else {
                        // It's a simple identifier, treat as shorthand
                        input.advance_to(&fork);
                        let ident = expr_path.path.segments[0].ident.clone();
                        Ok(Arg {
                            value: expr,
                            kind: ArgKind::Named(ident),
                        })
                    }
                } else {
                    // Not a path expression, treat as positional
                    input.advance_to(&fork);
                    Ok(Arg {
                        value: expr,
                        kind: ArgKind::Positional,
                    })
                }
            }
            Err(_) => {
                // If we can't parse it as an expression at all, that's an error
                Err(input.error("expected argument"))
            }
        }
    }
}
