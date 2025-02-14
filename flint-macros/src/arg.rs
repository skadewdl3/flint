use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Result, Token,
};

#[derive(Debug, Clone)]
pub enum ArgKind {
    Positional,
    Named(Ident),
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub value: Expr,
    pub kind: ArgKind,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        //
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
