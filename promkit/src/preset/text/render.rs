use std::cell::RefCell;

use promkit_core::{pane::Pane, PaneFactory};

use promkit_widgets::text;

use crate::{crossterm::event::Event, switch::ActiveKeySwitcher, PromptSignal};

use super::keymap;

pub struct Renderer {
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    pub text_state: text::State,
}

impl crate::Finalizer for Renderer {
    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
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
