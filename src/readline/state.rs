use std::fmt;

use crate::{
    crossterm::{style, terminal},
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    suggest::Suggest,
    Result,
};

/// Edit mode.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

/// Readline specific state.
#[derive(Debug)]
pub struct State {
    pub editor: Buffer,
    pub prev: Buffer,
    pub next: Buffer,
    pub title_lines: u16,
    /// A label as prompt (e.g. ">>").
    pub label: Graphemes,
    pub label_color: style::Color,
    /// A char to mask the input chars (e.g. "*"),
    /// for example when you type the passwords.
    pub mask: Option<Grapheme>,
    pub edit_mode: Mode,
    /// How many lines to receive the user input string.
    pub num_lines: Option<u16>,
    pub hstr: Option<History>,
    pub suggest: Option<Suggest>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.data)
    }
}

impl State {
    pub fn buffer_lines(&self) -> Result<u16> {
        let left_space = terminal::size()?.1 - self.title_lines;
        Ok(self.num_lines.unwrap_or(left_space))
    }

    pub fn buffer_limit(&self) -> Result<u16> {
        Ok(terminal::size()?.0 * self.buffer_lines()? - self.label.width() as u16 - 1)
    }
}
