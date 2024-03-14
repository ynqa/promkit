use std::any::type_name;

use crate::{
    impl_as_any, impl_cast, keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, text,
    AsAny, Error, Result,
};

pub struct Renderer {
    pub keymap: KeymapManager<Self>,
    pub title_snapshot: Snapshot<text::Renderer>,
    pub listbox_snapshot: Snapshot<listbox::Renderer>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.extend(self.title_snapshot.create_panes(width));
        panes.extend(self.listbox_snapshot.create_panes(width));
        panes
    }
}
