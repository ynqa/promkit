use std::cell::RefCell;

use promkit_core::{Pane, PaneFactory};

use promkit_widgets::{cursor::Cursor, text_editor};

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    switch::ActiveKeySwitcher,
    PromptSignal,
};

use super::keymap;

/// Represents the visual styles for different states of text editor components.
pub struct Style {
    /// Style for the prefix of the text editor.
    pub prefix_style: ContentStyle,
    /// Style for the character that is currently active (e.g., where the cursor is).
    pub active_char_style: ContentStyle,
    /// Style for characters that are not currently active.
    pub inactive_char_style: ContentStyle,
}

/// Manages rendering logic for text editors, including handling of styles and key mappings.
pub struct Renderer {
    /// A mutable reference to a key switcher that manages active key mappings.
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    /// Cursor managing the state of multiple text editors.
    pub text_editor_states: Cursor<Vec<text_editor::State>>,
    /// Default styles applied to text editors.
    pub default_styles: Vec<Style>,
    /// Styles applied to text editors when they are unselected.
    pub overwrite_styles: Vec<Style>,
}

impl crate::Finalizer for Renderer {
    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .text_editor_states
            .contents()
            .iter()
            .map(|state| state.texteditor.text_without_cursor().to_string())
            .collect())
    }
}

impl Renderer {
    /// Updates the styles of text editor states based on their active or inactive status.
    pub fn overwrite_styles(&mut self) {
        let current_position = self.text_editor_states.position();
        self.text_editor_states
            .contents_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, state)| {
                if i == current_position {
                    state.prefix_style = self.default_styles[i].prefix_style;
                    state.inactive_char_style = self.default_styles[i].inactive_char_style;
                    state.active_char_style = self.default_styles[i].active_char_style;
                } else {
                    state.prefix_style = self.overwrite_styles[i].prefix_style;
                    state.inactive_char_style = self.overwrite_styles[i].inactive_char_style;
                    state.active_char_style = self.overwrite_styles[i].active_char_style;
                }
            });
    }
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        self.text_editor_states
            .contents()
            .iter()
            .map(|state| state.create_pane(width, height))
            .collect()
    }

    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal> {
        let keymap = *self.keymap.borrow_mut().get();
        let signal = keymap(event, self);
        self.overwrite_styles();
        signal
    }
}
