//! This module provides a procedural macro for generating UI widget code.

/// Internal module for argument handling
mod arg;
/// Internal module for code generation
mod codegen;
/// Internal module for widget definitions
mod widget;

use codegen::WidgetHandlerOptions;
use proc_macro::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Expr, Result, Token,
};
use widget::{Widget, WidgetRenderer};

#[derive(Debug)]
enum MacroInput {
    Ui {
        widget: Widget,
        renderer: WidgetRenderer,
    },
    Raw {
        widget: Widget,
    },
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // First check if we start with a brace (ui_raw case)
        if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            let widget = content.parse()?;
            return Ok(MacroInput::Raw { widget });
        }

        // Otherwise, we're in a ui! case. Try parsing the renderer first
        let renderer = if input.peek(token::Paren) {
            // (area, buffer) case
            let content;
            syn::parenthesized!(content in input);

            let area = content.parse::<Expr>()?;
            content.parse::<Token![,]>()?;
            let buffer = content.parse::<Expr>()?;

            WidgetRenderer::Area { area, buffer }
        } else {
            // frame case
            let frame = input.parse::<Expr>()?;
            WidgetRenderer::Frame(frame)
        };

        // Both ui! cases require => { widget }
        input.parse::<Token![=>]>()?;

        let content;
        braced!(content in input);
        let widget = content.parse()?;

        Ok(MacroInput::Ui { widget, renderer })
    }
}

/// Generates UI widget code based on the macro input
///
/// # Arguments
///
/// * `input` - The input TokenStream containing the macro arguments
///
/// # Returns
///
/// TokenStream containing the generated widget code
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(input as MacroInput);
    if let MacroInput::Ui { ref widget, .. } = macro_input {
        let options = WidgetHandlerOptions::new(true, 0, 0, &macro_input, true);
        let output = codegen::generate_widget_code(widget, &options);
        output.into()
    } else {
        panic!("Cannot pass a widget directly to the ui!() macro")
    }
}

#[proc_macro]
pub fn widget(input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(input as MacroInput);
    if let MacroInput::Raw { ref widget, .. } = macro_input {
        let options = WidgetHandlerOptions::new(false, 0, 0, &macro_input, false);
        let output = codegen::generate_widget_code(widget, &options);
        output.into()
    } else {
        panic!("Cannot pass rendering options directly to the widget!() macro")
    }
}
