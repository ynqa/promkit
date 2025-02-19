use std::cell::RefCell;

use crate::{
    crossterm::event::Event, pane::Pane, switch::ActiveKeySwitcher, text, PaneFactory, PromptSignal,
};

use super::keymap;

pub struct Renderer {
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    pub text_state: text::State,
}

impl crate::Finalizer for Renderer {
    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.text_state.text.clone())
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![self.text_state.create_pane(width, height)]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
