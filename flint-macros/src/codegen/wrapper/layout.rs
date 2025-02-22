
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    widgets::{Widget, WidgetRef},
};

pub struct LayoutWrapper<'a> {
    layout: Layout,
    children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>,
}

impl<'a> LayoutWrapper<'a> {
    pub fn new(layout: Layout, children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>) -> Self {
        Self { layout, children }
    }
}

impl<'a> Widget for LayoutWrapper<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (idx, render_fn) in self.children.into_iter().enumerate() {
            render_fn(chunks[idx], buf);
        }
    }
}

impl<'a> WidgetRef for LayoutWrapper<'a> {
    fn render_ref<'b>(&'b self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (idx, render_fn) in self.children.iter().enumerate() {
            render_fn(chunks[idx], buf);
        }
    }
}
