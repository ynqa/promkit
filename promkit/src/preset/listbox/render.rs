use std::cell::RefCell;

use crate::{
    crossterm::event::Event, impl_as_any, listbox, pane::Pane, snapshot::Snapshot,
    switch::ActiveKeySwitcher, text, PaneFactory, PromptSignal, Result,
};

use super::keymap;

pub struct Renderer {
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    pub title_snapshot: Snapshot<text::State>,
    pub listbox_snapshot: Snapshot<listbox::State>,
}

impl_as_any!(Renderer);

impl crate::Renderer for Renderer {
    type Return = String;

    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width),
            self.listbox_snapshot.create_pane(width),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }

    fn finalize(&self) -> Result<Self::Return> {
        Ok(self.listbox_snapshot.after().listbox.get())
    }
}
