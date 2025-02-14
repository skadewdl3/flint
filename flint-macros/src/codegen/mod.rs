//! Module for handling widget code generation and related functionality.

pub mod conditional;
pub mod constructor;
pub mod layout;
pub mod util;
pub mod variable;

use crate::widget::{Widget, WidgetKind};
use conditional::handle_conditional_widget;
use constructor::handle_constructor_widget;
use layout::handle_layout_widget;
use syn::Ident;
use variable::handle_variable_widget;

/// Options for configuring widget code generation.
#[derive(Debug, Clone)]
pub struct WidgetHandlerOptions<'a> {
    /// Whether this widget is at the top level of the hierarchy.
    is_top_level: bool,
    /// ID of this widget's parent widget.
    parent_id: usize,
    /// Index of this widget among its siblings.
    child_index: usize,
    /// Identifier for the frame being rendered to.
    frame: &'a Ident,
}

impl<'a> WidgetHandlerOptions<'a> {
    /// Creates a new `WidgetHandlerOptions` with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `is_top_level` - Whether this widget is at the top level
    /// * `parent_id` - ID of the parent widget
    /// * `child_index` - Index among siblings
    /// * `frame` - Frame identifier
    pub fn new(is_top_level: bool, parent_id: usize, child_index: usize, frame: &'a Ident) -> Self {
        Self {
            is_top_level,
            parent_id,
            child_index,
            frame,
        }
    }
}

/// Generates code for rendering a widget based on its kind and options.
///
/// # Arguments
///
/// * `widget` - The widget to generate code for
/// * `options` - Configuration options for code generation
///
/// # Returns
///
/// A TokenStream containing the generated code
pub fn generate_widget_code(
    widget: &Widget,
    options: &WidgetHandlerOptions,
) -> proc_macro2::TokenStream {
    match &widget.kind {
        WidgetKind::Conditional {
            condition,
            if_child,
            else_child,
        } => handle_conditional_widget(condition, if_child, else_child, options),

        WidgetKind::Variable { expr } => handle_variable_widget(expr, options),

        WidgetKind::Constructor { name, constructor } => {
            handle_constructor_widget(&widget, name, constructor, options)
        }

        WidgetKind::Layout { name, children } => {
            handle_layout_widget(widget, name, children, options)
        }
    }
}
