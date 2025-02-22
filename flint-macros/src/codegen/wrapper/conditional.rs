use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// A widget wrapper that conditionally renders one of two widgets based on a boolean condition.
///
/// This wrapper allows for conditional rendering logic to be encapsulated in a widget.
/// When the condition is true, the "if" widget is rendered. When false, the optional
/// "else" widget is rendered if present.
pub struct ConditionalWrapper<W, E>
where
    W: Widget,
    E: Widget,
{
    if_widget: W,
    else_widget: Option<E>,
    condition: bool,
}

impl<W, E> ConditionalWrapper<W, E>
where
    W: Widget,
    E: Widget,
{
    /// Creates a new ConditionalWrapper with the given widget and condition.
    ///
    /// # Arguments
    ///
    /// * `if_widget` - The widget to render when the condition is true
    /// * `condition` - The boolean condition determining which widget to render
    pub fn new(if_widget: W, condition: bool) -> Self {
        Self {
            if_widget,
            else_widget: None,
            condition,
        }
    }

    /// Creates a new ConditionalWrapper with both an "if" widget and an "else" widget.
    ///
    /// # Arguments
    ///
    /// * `if_widget` - The widget to render when the condition is true
    /// * `else_widget` - The widget to render when the condition is false
    /// * `condition` - The boolean condition determining which widget to render
    pub fn with_else(if_widget: W, else_widget: E, condition: bool) -> Self {
        Self {
            if_widget,
            else_widget: Some(else_widget),
            condition,
        }
    }
}

impl<W, E> Widget for ConditionalWrapper<W, E>
where
    W: Widget,
    E: Widget,
{
    /// Renders either the "if" widget or "else" widget based on the condition.
    ///
    /// # Arguments
    ///
    /// * `area` - The area in which to render the widget
    /// * `buf` - The buffer to render to
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.condition {
            self.if_widget.render(area, buf);
        } else if let Some(else_widget) = self.else_widget {
            else_widget.render(area, buf);
        }
    }
}
