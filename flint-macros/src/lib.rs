//! This module provides a procedural macro for generating UI widget code.
//! It includes functionality for handling UI widgets and their rendering options.

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

/// Represents the different types of macro input that can be processed
///
/// # Variants
///
/// * `Ui` - Contains both a widget and its renderer
/// * `Raw` - Contains just a widget without rendering information
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

/// Implements the parsing logic for MacroInput
///
/// # Arguments
///
/// * `input` - The ParseStream to read tokens from
///
/// # Returns
///
/// Result containing the parsed MacroInput or an error
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

/// Generates UI widget code with rendering options
///
/// This macro processes input containing both widget definition and rendering information.
///
/// # Arguments
///
/// * `input` - The input TokenStream containing the macro arguments
///
/// # Returns
///
/// TokenStream containing the generated widget code
///
/// # Panics
///
/// Panics if a widget is passed directly without rendering options
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(input as MacroInput);
    if let MacroInput::Ui { ref widget, .. } = macro_input {
        let options = WidgetHandlerOptions::new(true, 0, 0, &macro_input);
        let output = codegen::generate_widget_code(widget, &options);
        output.into()
    } else {
        panic!("Cannot pass a widget directly to the ui!() macro")
    }
}

/// Generates widget code without rendering options
///
/// This macro processes raw widget definitions without any rendering information.
///
/// # Arguments
///
/// * `input` - The input TokenStream containing the widget definition
///
/// # Returns
///
/// TokenStream containing the generated widget code
///
/// # Panics
///
/// Panics if rendering options are included in the input
#[proc_macro]
pub fn widget(input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(input as MacroInput);
    if let MacroInput::Raw { ref widget, .. } = macro_input {
        let options = WidgetHandlerOptions::new(true, 0, 0, &macro_input);
        let output = codegen::generate_widget_code(widget, &options);
        output.into()
    } else {
        panic!("Cannot pass rendering options directly to the widget!() macro")
    }
}
