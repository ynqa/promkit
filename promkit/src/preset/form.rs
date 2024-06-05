use std::cell::RefCell;

use render::DefaultStyle;

use crate::{core::Cursor, switch::ActiveKeySwitcher, text_editor, Prompt};

mod keymap;
mod render;

pub struct Form {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    text_editor_states: Vec<text_editor::State>,
}

impl Form {
    pub fn new<I: IntoIterator<Item = text_editor::State>>(states: I) -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default as keymap::Keymap),
            text_editor_states: states.into_iter().collect(),
        }
    }

    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        let default_styles = self
            .text_editor_states
            .iter()
            .map(|state| DefaultStyle {
                prefix_style: state.prefix_style,
                active_char_style: state.active_char_style,
                inactive_char_style: state.inactive_char_style,
            })
            .collect();
        let mut renderer = render::Renderer {
            keymap: RefCell::new(self.keymap),
            text_editor_states: Cursor::new(self.text_editor_states, 0, false),
            default_styles,
        };
        renderer.overwrite_styles();
        Ok(Prompt { renderer })
    }
}
