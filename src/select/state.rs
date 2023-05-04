use std::fmt;

use crate::{
    crossterm::{style, terminal},
    grapheme::Graphemes,
    grid::UpstreamContext,
    internal::selector::Selector,
    Result,
};

/// Select specific state.
pub struct State {
    pub editor: Selector,
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
    pub fn selector_lines(&self, context: &UpstreamContext) -> Result<u16> {
        let unused_rows = terminal::size()?.1 - context.used_rows;
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
