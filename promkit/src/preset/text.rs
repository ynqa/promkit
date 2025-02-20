use std::{cell::RefCell, io};

use crossterm::style::ContentStyle;

use crate::{switch::ActiveKeySwitcher, text, Prompt};

pub mod keymap;
pub mod render;

pub struct Text {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    text_state: text::State,
    /// Writer to which promptkit write its contents
    writer: Box<dyn io::Write>,
}

impl Text {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
            text_state: text::State {
                text: text::Text::from(text),
                style: Default::default(),
                lines: None,
            },
            writer: Box::new(io::stdout()),
        }
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.text_state.style = style;
        self
    }

    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                text_state: self.text_state,
            },
            writer: self.writer,
        })
    }
}
