use std::fmt;

use crate::{
    crossterm::{style, terminal},
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    readline::Mode,
    suggest::Suggest,
    Result,
};

/// Readline specific state.
#[derive(Debug)]
pub struct State {
    pub editor: Buffer,
    pub prev: Buffer,
    pub next: Buffer,
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
    pub fn buffer_lines(&self, unused_rows: u16) -> Result<u16> {
        Ok(*vec![unused_rows, self.num_lines.unwrap_or(unused_rows)]
            .iter()
            .min()
            .unwrap_or(&unused_rows))
    }

    pub fn buffer_limit(&self, unused_rows: u16) -> Result<u16> {
        // -1 is for the space for cursor.
        Ok(terminal::size()?.0 * self.buffer_lines(unused_rows)? - self.label.width() as u16 - 1)
    }
}
