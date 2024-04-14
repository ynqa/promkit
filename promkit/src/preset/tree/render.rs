use crate::{
    impl_as_any, impl_cast, pane::Pane, snapshot::Snapshot, switch::ActiveKeySwitcher, text, tree,
    PaneFactory,
};

use super::keymap;

/// A `Renderer` responsible for rendering the tree structure.
/// It manages key mappings, title, and tree renderings.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// Snapshot of the title renderer.
    pub title_snapshot: Snapshot<text::State>,
    /// Snapshot of the tree renderer.
    pub tree_snapshot: Snapshot<tree::State>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width),
            self.tree_snapshot.create_pane(width),
        ]
    }
}
