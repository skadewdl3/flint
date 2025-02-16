use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    widgets::Widget,
};

pub struct LayoutWrapper<'a> {
    layout: Layout,
    children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>,
}

impl LayoutWrapper<'_> {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            children: Vec::new(),
        }
    }

    pub fn push(mut self, child: impl Fn(Rect, &mut Buffer) + 'static) -> Self {
        self.children.push(Box::new(child));
        self
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
