use proc_macro2::TokenStream;
use quote::quote;

use crate::widget::Widget;

/// Generates a unique identifier using an atomic counter.
///
/// Returns an incrementing u32 value that is guaranteed to be unique
/// across all threads. Uses a relaxed ordering which provides minimal
/// synchronization guarantees but maximizes performance.
///
/// This function is used by LayoutWidgets to generate unique identifiers for
/// each layout (to be used as the variable name). It ensures that each layout
/// has a unique identifier, and the user doesn't have to worry about any of them.
pub fn generate_unique_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn get_render_function(widget: &Widget) -> (TokenStream, TokenStream) {
    let render_fn = match widget.render_ref {
        true => quote! { ratatui::widgets::WidgetRef::render_ref },
        false => quote! { ratatui::widgets::Widget::render },
    };

    let frame_render_fn = match widget.render_ref {
        true => quote! { render_widget_ref },
        false => quote! { render_widget },
    };
    (render_fn, frame_render_fn)
}

pub fn get_stateful_render_function(widget: &Widget) -> (TokenStream, TokenStream) {
    let render_fn = match widget.render_ref {
        true => quote! { ratatui::widgets::StatefulWidgetRef::render_ref },
        false => quote! { ratatui::widgets::StatefulWidget::render },
    };

    let frame_render_fn = match widget.render_ref {
        true => quote! { render_stateful_widget_ref },
        false => quote! { render_stateful_widget },
    };
    (render_fn, frame_render_fn)
}
