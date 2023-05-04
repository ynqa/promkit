use crate::{crossterm::style, grapheme::Graphemes, termutil, Result};

/// Text specific state.
#[derive(Debug)]
pub struct State {
    pub text: Graphemes,
    pub text_color: style::Color,
}

impl Default for State {
    fn default() -> Self {
        Self {
            text: Default::default(),
            text_color: style::Color::Reset,
        }
    }
}

impl State {
    pub fn text_lines(&self) -> Result<u16> {
        termutil::num_lines(&self.text)
    }
}
