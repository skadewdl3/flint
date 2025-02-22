use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    widgets::Widget,
};

/// A widget that wraps an iterator and renders each item according to a provided layout
///
/// This widget takes an iterator of items and renders each item in its own section
/// defined by splitting an area according to the provided layout configuration.
pub struct IterLayoutWrapper<'a, I>
where
    I: Iterator,
{
    /// The layout configuration used to split the rendering area
    layout: Layout,
    /// The iterator containing items to render
    iterator: I,
    /// Function that defines how to render each item in its allocated space
    render_fn: Box<dyn Fn(I::Item, &Rect, &mut Buffer) + 'a>,
}

impl<'a, I> IterLayoutWrapper<'a, I>
where
    I: Iterator,
{
    /// Creates a new IterLayoutWrapper with the given layout, iterator and render function
    ///
    /// # Arguments
    ///
    /// * `layout` - Layout configuration that defines how to split the rendering area
    /// * `iterator` - Iterator containing the items to render
    /// * `render_fn` - Function that will be called to render each item
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
    /// Renders the widget by splitting the area according to layout and rendering each item
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (chunk, item) in chunks.into_iter().zip(self.iterator) {
            (self.render_fn)(item, chunk, buf);
        }
    }
}
