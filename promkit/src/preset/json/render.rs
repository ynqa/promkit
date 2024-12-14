use std::cell::RefCell;

use crate::{
    crossterm::event::Event, jsonstream, pane::Pane, switch::ActiveKeySwitcher, text, PaneFactory,
    PromptSignal,
};

use super::keymap;

/// A `Renderer` responsible for rendering JSON presets.
/// It manages key mappings, title, and JSON content rendering.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// A renderer used for the title.
    pub title_state: text::State,
    /// A renderer used for JSON content.
    pub json_state: jsonstream::State,
}

impl crate::Finalizer for Renderer {
    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.title_state.create_pane(width, height),
            self.json_state.create_pane(width, height),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
