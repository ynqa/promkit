use std::cell::RefCell;

use promkit_core::{Pane, PaneFactory};

use promkit_widgets::{text, tree};

use crate::{crossterm::event::Event, switch::ActiveKeySwitcher, PromptSignal};

use super::keymap;

/// A `Renderer` responsible for rendering the tree structure.
/// It manages key mappings, title, and tree renderings.
pub struct Renderer {
    /// Manages key mappings specific to this renderer.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// Snapshot of the title renderer.
    pub title_state: text::State,
    /// Snapshot of the tree renderer.
    pub tree_state: tree::State,
}

impl crate::Finalizer for Renderer {
    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.tree_state.tree.get())
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.title_state.create_pane(width, height),
            self.tree_state.create_pane(width, height),
        ]
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        keymap(event, self)
    }
}
