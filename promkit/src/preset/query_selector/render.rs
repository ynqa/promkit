use std::cell::RefCell;

use crate::{
    crossterm::event::Event,
    impl_as_any,
    listbox::{self, Listbox},
    pane::Pane,
    snapshot::Snapshot,
    switch::ActiveKeySwitcher,
    text, text_editor, PaneFactory, PromptSignal, Result,
};

/// Used to process and filter a list of options
/// based on the input text in the `QuerySelector` component.
pub type Filter = fn(&str, &Vec<String>) -> Vec<String>;

use super::keymap;

/// Represents a renderer for the query selector.
/// This struct manages the rendering process of different components within the query selector,
/// including key mappings, title, text editor, and listbox.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// Snapshot of the title renderer.
    pub title_snapshot: Snapshot<text::State>,
    /// Snapshot of the text editor renderer.
    pub text_editor_snapshot: Snapshot<text_editor::State>,
    /// Snapshot of the listbox renderer.
    pub listbox_snapshot: Snapshot<listbox::State>,
    pub filter: Filter,
}

impl_as_any!(Renderer);

impl crate::Renderer for Renderer {
    type Return = String;

    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.title_snapshot.create_pane(width),
            self.text_editor_snapshot.create_pane(width),
            self.listbox_snapshot.create_pane(width),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        let signal = keymap(event, self);
        if self.text_editor_snapshot.after().texteditor.text()
            != self.text_editor_snapshot.borrow_before().texteditor.text()
        {
            let query = self
                .text_editor_snapshot
                .after()
                .texteditor
                .text_without_cursor()
                .to_string();

            let list = (self.filter)(&query, self.listbox_snapshot.init().listbox.items());
            self.listbox_snapshot.after_mut().listbox = Listbox::from_iter(list);
        }
        signal
    }

    fn finalize(&self) -> Result<Self::Return> {
        Ok(self.listbox_snapshot.after().listbox.get())
    }
}
