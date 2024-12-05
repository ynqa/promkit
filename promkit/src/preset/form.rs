use std::{cell::RefCell, io};

use crate::{
    core::Cursor,
    crossterm::style::{Attribute, Attributes},
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text_editor, Prompt,
};

mod keymap;
mod render;

/// `Form` struct provides functionality for managing multiple text input fields.
pub struct Form {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    text_editor_states: Vec<text_editor::State>,
    /// Overwrite the default styles of text editor states when unselected.
    overwrite_styles: Vec<render::Style>,
    /// Writer to which promptkit write its contents
    writer: Box<dyn io::Write>,
}

impl Form {
    pub fn new<I: IntoIterator<Item = text_editor::State>>(states: I) -> Self {
        let (text_editor_states, overwrite_styles): (Vec<_>, Vec<_>) = states
            .into_iter()
            .map(|state| {
                let style = render::Style {
                    prefix_style: StyleBuilder::from(state.prefix_style)
                        .attrs(Attributes::from(Attribute::Dim))
                        .build(),
                    inactive_char_style: StyleBuilder::from(state.inactive_char_style)
                        .attrs(Attributes::from(Attribute::Dim))
                        .build(),
                    active_char_style: StyleBuilder::new()
                        .attrs(Attributes::from(Attribute::Dim))
                        .build(),
                };
                (state, style)
            })
            .unzip();
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default as keymap::Keymap),
            text_editor_states,
            overwrite_styles,
            writer: Box::new(io::stdout()),
        }
    }

    /// Sets writer.
    pub fn writer<W: io::Write + 'static>(mut self, writer: W) -> Self {
        self.writer = Box::new(writer);
        self
    }

    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        let default_styles = self
            .text_editor_states
            .iter()
            .map(|state| render::Style {
                prefix_style: state.prefix_style,
                active_char_style: state.active_char_style,
                inactive_char_style: state.inactive_char_style,
            })
            .collect();
        let mut renderer = render::Renderer {
            keymap: RefCell::new(self.keymap),
            text_editor_states: Cursor::new(self.text_editor_states, 0, false),
            default_styles,
            overwrite_styles: self.overwrite_styles,
        };
        renderer.overwrite_styles();
        Ok(Prompt {
            renderer,
            writer: self.writer,
        })
    }
}
