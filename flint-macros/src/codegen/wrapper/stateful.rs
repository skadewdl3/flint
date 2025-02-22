use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};
use std::cell::RefCell;

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
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.borrow_mut();
        ratatui::widgets::StatefulWidget::render(self.widget, area, buf, &mut *state);
    }
}
