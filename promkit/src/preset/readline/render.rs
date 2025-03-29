use std::cell::RefCell;

use promkit_core::{pane::Pane, PaneFactory};

use promkit_widgets::{listbox, text, text_editor};

use crate::{
    crossterm::event::Event, snapshot::Snapshot, suggest::Suggest, switch::ActiveKeySwitcher,
    validate::ValidatorManager, PromptSignal,
};

use super::keymap;

/// A `Renderer` for the readline preset, responsible for managing the rendering process.
/// It holds references to various components and their states, facilitating the rendering of the readline interface.
pub struct Renderer {
    /// Manages key bindings and their associated actions within the readline interface.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// Holds a title's renderer state, used for rendering the title section.
    pub title_state: text::State,
    /// Holds a snapshot of the text editor's renderer state, used for rendering the text input area.
    pub text_editor_snapshot: Snapshot<text_editor::State>,
    /// Optional suggest component for autocomplete functionality.
    pub suggest: Option<Suggest>,
    /// Holds a snapshot of the suggest box's renderer state, used when rendering suggestions for autocomplete.
    pub suggest_snapshot: Snapshot<listbox::State>,
    /// Optional validator manager for input validation.
    pub validator: Option<ValidatorManager<str>>,
    /// Holds a snapshot of the error message's renderer state, used for rendering error messages.
    pub error_message_snapshot: Snapshot<text::State>,
}

impl crate::Finalizer for Renderer {
    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        let ret = self
            .text_editor_snapshot
            .after()
            .texteditor
            .text_without_cursor()
            .to_string();

        // Keep history over state reset
        let history = self.text_editor_snapshot.after_mut().history.take();
        self.text_editor_snapshot.reset_after_to_init();
        self.text_editor_snapshot.after_mut().history = history;

        Ok(ret)
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.title_state.create_pane(width, height),
            self.error_message_snapshot.create_pane(width, height),
            self.text_editor_snapshot.create_pane(width, height),
            self.suggest_snapshot.create_pane(width, height),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
