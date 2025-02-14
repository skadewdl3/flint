use syn::{
    parse::{Parse, ParseStream},
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
    /// - A positional argument which is just a value
    ///
    /// # Returns
    /// - `Ok(Arg)` if parsing succeeds
    /// - `Err(Error)` if parsing fails
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
