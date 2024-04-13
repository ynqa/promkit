use crate::{
    impl_as_any, impl_cast, keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, text,
    text_editor, PaneFactory,
};

/// Represents a renderer for the query selector.
/// This struct manages the rendering process of different components within the query selector,
/// including key mappings, title, text editor, and listbox.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: KeymapManager<Self>,
    /// Snapshot of the title renderer.
    pub title_snapshot: Snapshot<text::State>,
    /// Snapshot of the text editor renderer.
    pub text_editor_snapshot: Snapshot<text_editor::State>,
    /// Snapshot of the listbox renderer.
    pub listbox_snapshot: Snapshot<listbox::State>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.push(self.title_snapshot.create_pane(width));
        panes.push(self.text_editor_snapshot.create_pane(width));
        panes.push(self.listbox_snapshot.create_pane(width));
        panes
    }
}
