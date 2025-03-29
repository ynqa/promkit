use std::cell::RefCell;

use promkit_core::{pane::Pane, PaneFactory};

use promkit_widgets::{checkbox, text};

use crate::{crossterm::event::Event, switch::ActiveKeySwitcher, PromptSignal};

use super::keymap;

/// A `Renderer` for rendering checkbox presets.
///
/// This struct is responsible for managing the rendering process of a checkbox preset,
/// including handling keymaps, and managing snapshots of the title and checkbox states.
pub struct Renderer {
    /// Manages key mappings for the renderer.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// A title's renderer state.
    pub title_state: text::State,
    /// A checkbox's renderer state.
    pub checkbox_state: checkbox::State,
}

impl crate::Finalizer for Renderer {
    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .checkbox_state
            .checkbox
            .get()
            .iter()
            .map(|e| e.to_string())
            .collect())
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.title_state.create_pane(width, height),
            self.checkbox_state.create_pane(width, height),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
