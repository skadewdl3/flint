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
        // First, check for named parameter pattern
        if input.peek(Ident) && input.peek2(Token![:]) {
            let name = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let value = input.parse::<Expr>()?;
            return Ok(Arg {
                value,
                kind: ArgKind::Named(name),
            });
        }

        // If not named, try parsing as a regular expression
        let value = input.parse::<Expr>()?;
        Ok(Arg {
            value,
            kind: ArgKind::Positional,
        })
    }
}
