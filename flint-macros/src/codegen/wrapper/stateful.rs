use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};
use std::cell::RefCell;

/// A wrapper struct that manages state for a stateful widget
///
/// This provides a way to combine a widget that requires state with its state, allowing the
/// widget to be used in contexts that expect a regular Widget trait object.
///
/// # Type Parameters
///
/// * `'a` - The lifetime of the state reference
/// * `W` - The widget type that implements StatefulWidget
/// * `S` - The state type used by the widget
pub struct StatefulWrapper<'a, W, S>
where
    W: StatefulWidget<State = S>,
{
    widget: W,
    state: RefCell<&'a mut S>,
}

impl<'a, W, S> StatefulWrapper<'a, W, S>
where
    W: StatefulWidget<State = S>,
{
    /// Creates a new StatefulWrapper with the given widget and state
    ///
    /// # Arguments
    ///
    /// * `widget` - The stateful widget to wrap
    /// * `state` - Mutable reference to the state for the widget
    pub fn new(widget: W, state: &'a mut S) -> Self {
        Self {
            widget,
            state: RefCell::new(state),
        }
    }
}

impl<'a, W, S> Widget for StatefulWrapper<'a, W, S>
where
    W: StatefulWidget<State = S>,
{
    /// Renders the wrapped widget with its state
    ///
    /// # Arguments
    ///
    /// * `area` - The area to render the widget in
    /// * `buf` - The buffer to render to
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.borrow_mut();
        ratatui::widgets::StatefulWidget::render(self.widget, area, buf, &mut *state);
    }
}
