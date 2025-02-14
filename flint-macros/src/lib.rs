mod arg;
mod codegen;
mod widget;

use codegen::WidgetHandlerOptions;
use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Result, Token,
};
use widget::Widget;

#[derive(Debug)]
struct UiMacroInput {
    frame: Ident,
    widget: Widget,
}

impl Parse for UiMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let frame = input.parse()?;
        input.parse::<Token![=>]>()?;
        let widget = input.parse()?;

        Ok(UiMacroInput { frame, widget })
    }
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let UiMacroInput { frame, widget, .. } = parse_macro_input!(input as UiMacroInput);
    let options = WidgetHandlerOptions::new(true, 0, 0, &frame);
    let output = codegen::generate_widget_code(&widget, &options);
    output.into()
}
