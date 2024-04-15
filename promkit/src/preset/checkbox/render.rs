use std::cell::RefCell;

use crate::{
    checkbox, crossterm::event::Event, impl_as_any, pane::Pane, snapshot::Snapshot,
    switch::ActiveKeySwitcher, text, PaneFactory, PromptSignal,
};

use super::keymap;

/// A `Renderer` for rendering checkbox presets.
///
/// This struct is responsible for managing the rendering process of a checkbox preset,
/// including handling keymaps, and managing snapshots of the title and checkbox states.
pub struct Renderer {
    /// Manages key mappings for the renderer.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// A snapshot of the title's renderer state.
    pub title_snapshot: Snapshot<text::State>,
    /// A snapshot of the checkbox's renderer state.
    pub checkbox_snapshot: Snapshot<checkbox::State>,
}

impl_as_any!(Renderer);

impl crate::Finalizer for Renderer {
    type Return = Vec<String>;

    fn finalize(&self) -> anyhow::Result<Self::Return> {
        Ok(self.checkbox_snapshot.after().checkbox.get())
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width, height),
            self.checkbox_snapshot.create_pane(width, height),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
