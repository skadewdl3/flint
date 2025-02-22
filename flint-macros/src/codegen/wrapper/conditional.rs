use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

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
    /// Creates a new ConditionalWrapper with the given widget and condition
    pub fn new(if_widget: W, condition: bool) -> Self {
        Self {
            if_widget,
            else_widget: None,
            condition,
        }
    }

    /// Adds an else widget to the conditional wrapper
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
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.condition {
            self.if_widget.render(area, buf);
        } else if let Some(else_widget) = self.else_widget {
            else_widget.render(area, buf);
        }
    }
}
