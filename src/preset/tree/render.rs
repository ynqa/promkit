use std::any::type_name;

use crate::{
    impl_as_any, impl_cast, keymap::KeymapManager, pane::Pane, snapshot::Snapshot, text, tree,
    AsAny, Error, Result,
};

/// A `Renderer` responsible for rendering the tree structure.
/// It manages key mappings, title, and tree renderings.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: KeymapManager<Self>,
    /// Snapshot of the title renderer.
    pub title_snapshot: Snapshot<text::Renderer>,
    /// Snapshot of the tree renderer.
    pub tree_snapshot: Snapshot<tree::Renderer>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.extend(self.title_snapshot.create_panes(width));
        panes.extend(self.tree_snapshot.create_panes(width));
        panes
    }
}
