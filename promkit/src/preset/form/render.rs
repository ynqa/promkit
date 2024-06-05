use std::cell::RefCell;

use crate::{
    core::Cursor,
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, ContentStyle},
    },
    pane::Pane,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text_editor, PaneFactory, PromptSignal,
};

use super::keymap;

pub struct DefaultStyle {
    pub prefix_style: ContentStyle,
    pub active_char_style: ContentStyle,
    pub inactive_char_style: ContentStyle,
}

pub struct Renderer {
    pub keymap: RefCell<ActiveKeySwitcher<keymap::Keymap>>,
    pub text_editor_states: Cursor<Vec<text_editor::State>>,
    pub default_styles: Vec<DefaultStyle>,
}

impl crate::Finalizer for Renderer {
    type Return = Vec<String>;

    fn finalize(&self) -> anyhow::Result<Self::Return> {
        Ok(self
            .text_editor_states
            .contents()
            .iter()
            .map(|state| state.texteditor.text_without_cursor().to_string())
            .collect())
    }
}

impl Renderer {
    pub fn overwrite_styles(&mut self) {
        let current_position = self.text_editor_states.position();
        self.text_editor_states
            .contents_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(i, state)| {
                if i != current_position {
                    state.prefix_style = StyleBuilder::from(state.prefix_style)
                        .attrs(Attributes::from(Attribute::Dim))
                        .build();
                    state.inactive_char_style = StyleBuilder::from(state.inactive_char_style)
                        .attrs(Attributes::from(Attribute::Dim))
                        .build();
                    state.active_char_style = StyleBuilder::new()
                        .attrs(Attributes::from(Attribute::Dim))
                        .build();
                } else {
                    state.prefix_style = self.default_styles[i].prefix_style;
                    state.inactive_char_style = self.default_styles[i].inactive_char_style;
                    state.active_char_style = self.default_styles[i].active_char_style;
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
