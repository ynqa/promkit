use std::any::type_name;

use crate::{
    checkbox, impl_as_any, impl_cast, keymap::KeymapManager, pane::Pane, snapshot::Snapshot, text,
    AsAny, Error, Result,
};

/// A `Renderer` for rendering checkbox presets.
///
/// This struct is responsible for managing the rendering process of a checkbox preset,
/// including handling keymaps, and managing snapshots of the title and checkbox states.
pub struct Renderer {
    /// Manages key mappings for the renderer.
    pub keymap: KeymapManager<Self>,
    /// A snapshot of the title's renderer state.
    pub title_snapshot: Snapshot<text::Renderer>,
    /// A snapshot of the checkbox's renderer state.
    pub checkbox_snapshot: Snapshot<checkbox::Renderer>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.extend(self.title_snapshot.create_panes(width));
        panes.extend(self.checkbox_snapshot.create_panes(width));
        panes
    }
}
