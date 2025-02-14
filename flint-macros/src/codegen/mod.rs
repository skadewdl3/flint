use crate::widget::{Widget, WidgetKind};
use conditional::handle_conditional_widget;
use constructor::handle_constructor_widget;
use layout::handle_layout_widget;
use quote::quote;
use syn::Ident;

pub mod conditional;
pub mod constructor;
pub mod layout;
pub mod util;

#[derive(Debug, Clone)]
pub struct WidgetHandlerOptions<'a> {
    is_top_level: bool,
    parent_id: usize,
    child_index: usize,
    frame: &'a Ident,
}

impl<'a> WidgetHandlerOptions<'a> {
    pub fn new(is_top_level: bool, parent_id: usize, child_index: usize, frame: &'a Ident) -> Self {
        Self {
            is_top_level,
            parent_id,
            child_index,
            frame,
        }
    }
}

pub fn generate_widget_code(
    widget: &Widget,
    options: &WidgetHandlerOptions,
) -> proc_macro2::TokenStream {
    let WidgetHandlerOptions { is_top_level, .. } = options;

    match &widget.kind {
        WidgetKind::Conditional {
            condition,
            if_child,
            else_child,
        } => handle_conditional_widget(condition, if_child, else_child, options),

        WidgetKind::Variable { expr } => {
            if *is_top_level {
                quote! {
                    frame.render_widget(&#expr, frame.area());
                }
            } else {
                quote! { #expr }
            }
        }

        WidgetKind::Constructor { name, constructor } => {
            handle_constructor_widget(&widget, name, constructor, options)
        }

        WidgetKind::Layout { name, children } => {
            handle_layout_widget(widget, name, children, options)
        }
    }
}
