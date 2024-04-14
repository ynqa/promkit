use crate::{
    checkbox, impl_as_any, impl_cast, pane::Pane, snapshot::Snapshot, switch::ActiveKeySwitcher,
    text, PaneFactory,
};

use super::keymap;

/// A `Renderer` for rendering checkbox presets.
///
/// This struct is responsible for managing the rendering process of a checkbox preset,
/// including handling keymaps, and managing snapshots of the title and checkbox states.
pub struct Renderer {
    /// Manages key mappings for the renderer.
    pub keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// A snapshot of the title's renderer state.
    pub title_snapshot: Snapshot<text::State>,
    /// A snapshot of the checkbox's renderer state.
    pub checkbox_snapshot: Snapshot<checkbox::State>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width),
            self.checkbox_snapshot.create_pane(width),
        ]
    }
}
