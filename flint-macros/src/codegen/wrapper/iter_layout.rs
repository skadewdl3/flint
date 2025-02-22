
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    widgets::Widget,
};

pub struct IterLayoutWrapper<'a, I>
where
    I: Iterator,
{
    layout: Layout,
    iterator: I,
    render_fn: Box<dyn Fn(I::Item, &Rect, &mut Buffer) + 'a>,
}

impl<'a, I> IterLayoutWrapper<'a, I>
where
    I: Iterator,
{
    pub fn new<F>(layout: Layout, iterator: I, render_fn: F) -> Self
    where
        F: Fn(I::Item, &Rect, &mut Buffer) + 'a,
    {
        Self {
            layout,
            iterator,
            render_fn: Box::new(render_fn),
        }
    }
}

impl<'a, I> Widget for IterLayoutWrapper<'a, I>
where
    I: Iterator,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (chunk, item) in chunks.into_iter().zip(self.iterator) {
            (self.render_fn)(item, chunk, buf);
        }
    }
}
