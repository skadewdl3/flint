use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    widgets::{Widget, WidgetRef},
};

/// A wrapper around a [`Layout`] that manages a collection of child render functions.
///
/// This struct provides a way to combine a layout configuration with multiple rendering
/// functions that will be applied to the layout's chunks.
pub struct LayoutWrapper<'a> {
    /// The layout configuration that determines how the area is split
    layout: Layout,
    /// Collection of rendering functions to be applied to each layout chunk
    children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>,
}

impl<'a> LayoutWrapper<'a> {
    /// Creates a new `LayoutWrapper` with the specified layout and child render functions.
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout configuration to use for splitting the area
    /// * `children` - Vector of render functions to be applied to the layout chunks
    pub fn new(layout: Layout, children: Vec<Box<dyn Fn(Rect, &mut Buffer) + 'a>>) -> Self {
        Self { layout, children }
    }
}

impl<'a> Widget for LayoutWrapper<'a> {
    /// Renders the widget by consuming it, splitting the area according to the layout,
    /// and applying each child render function to its corresponding chunk.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (idx, render_fn) in self.children.into_iter().enumerate() {
            render_fn(chunks[idx], buf);
        }
    }
}

impl<'a> WidgetRef for LayoutWrapper<'a> {
    /// Renders the widget by reference, splitting the area according to the layout,
    /// and applying each child render function to its corresponding chunk.
    fn render_ref<'b>(&'b self, area: Rect, buf: &mut Buffer) {
        let chunks = self.layout.split(area);
        for (idx, render_fn) in self.children.iter().enumerate() {
            render_fn(chunks[idx], buf);
        }
    }
}
