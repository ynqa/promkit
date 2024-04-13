use crate::{
    impl_as_any, impl_cast, keymap::KeymapManager, listbox, pane::Pane, snapshot::Snapshot, text,
    PaneFactory,
};

pub struct Renderer {
    pub keymap: KeymapManager<Self>,
    pub title_snapshot: Snapshot<text::State>,
    pub listbox_snapshot: Snapshot<listbox::State>,
}

impl_as_any!(Renderer);
impl_cast!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let mut panes = Vec::new();
        panes.push(self.title_snapshot.create_pane(width));
        panes.push(self.listbox_snapshot.create_pane(width));
        panes
    }
}
