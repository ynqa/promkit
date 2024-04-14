use crate::{
    impl_as_any, impl_cast, listbox, pane::Pane, snapshot::Snapshot, switch::ActiveKeySwitcher,
    text, text_editor, PaneFactory,
};

use super::keymap;

/// Represents a renderer for the query selector.
/// This struct manages the rendering process of different components within the query selector,
/// including key mappings, title, text editor, and listbox.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: ActiveKeySwitcher<keymap::Keymap>,
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
        vec![
            self.title_snapshot.create_pane(width),
            self.text_editor_snapshot.create_pane(width),
            self.listbox_snapshot.create_pane(width),
        ]
    }
}
