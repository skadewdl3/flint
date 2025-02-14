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
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Ident, Result, Token,
};
use widget::Widget;

/// Input structure for the UI macro representing a widget and its containing frame
#[derive(Debug)]
struct UiMacroInput {
    /// The identifier of the frame containing the widget
    frame: Ident,
    /// The widget definition
    widget: Widget,
}

impl Parse for UiMacroInput {
    /// Parses the macro input into a UiMacroInput structure
    ///
    /// # Arguments
    ///
    /// * `input` - The parse stream containing the macro input
    ///
    /// # Returns
    ///
    /// Result containing the parsed UiMacroInput if successful
    fn parse(input: ParseStream) -> Result<Self> {
        let frame = match input.parse() {
            Ok(frame) => frame,
            Err(_) => return Err(input.error("Expected ratatui::Frame identifier")),
        };
        match input.parse::<Token![=>]>() {
            Ok(_) => (),
            Err(_) => return Err(input.error("Expected => between frame and UI")),
        };
        let widget = match input.parse() {
            Ok(widget) => widget,
            Err(_) => return Err(input.error("Unable to parse widget")),
        };

        Ok(UiMacroInput { frame, widget })
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
    let UiMacroInput { frame, widget, .. } = parse_macro_input!(input as UiMacroInput);
    let options = WidgetHandlerOptions::new(true, 0, 0, &frame);
    let output = codegen::generate_widget_code(&widget, &options);
    output.into()
}
