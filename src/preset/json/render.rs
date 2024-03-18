use crate::{
    impl_as_any, impl_cast, json, keymap::KeymapManager, pane::Pane, snapshot::Snapshot, text,
};

/// A `Renderer` responsible for rendering JSON presets.
/// It manages key mappings, title, and JSON content rendering.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: KeymapManager<Self>,
    /// Snapshot of the renderer used for the title.
    pub title_snapshot: Snapshot<text::Renderer>,
    /// Snapshot of the renderer used for JSON content.
    pub json_snapshot: Snapshot<json::Renderer>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.extend(self.title_snapshot.create_panes(width));
        panes.extend(self.json_snapshot.create_panes(width));
        panes
    }
}
