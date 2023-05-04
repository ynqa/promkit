use std::fmt;

use crate::{crossterm::style, grapheme::Graphemes, internal::selector::Selector, Result};

/// Select specific state.
pub struct State {
    pub editor: Selector,
    pub prev: Selector,
    pub next: Selector,
    /// A symbol to emphasize the selected item (e.g. ">").
    pub label: Graphemes,
    pub label_color: style::Color,
    pub screen_position: u16,
    pub window: Option<u16>,
    pub suffix_after_trim: Graphemes,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.get())
    }
}

impl State {
    pub fn selector_lines(&self, unused_rows: u16) -> Result<u16> {
        Ok(*vec![
            unused_rows,
            self.window.unwrap_or(unused_rows),
            self.editor.data.len() as u16,
        ]
        .iter()
        .min()
        .unwrap_or(&unused_rows))
    }
}
