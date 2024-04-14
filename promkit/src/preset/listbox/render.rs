use crate::{
    impl_as_any, impl_cast, listbox, pane::Pane, snapshot::Snapshot, switch::ActiveKeySwitcher,
    text, PaneFactory,
};

use super::keymap;

pub struct Renderer {
    pub keymap: ActiveKeySwitcher<keymap::Keymap>,
    pub title_snapshot: Snapshot<text::State>,
    pub listbox_snapshot: Snapshot<listbox::State>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width),
            self.listbox_snapshot.create_pane(width),
        ]
    }
}
