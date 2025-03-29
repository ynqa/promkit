use std::collections::HashSet;

use promkit_core::{
    PaneFactory, crossterm::style::ContentStyle, grapheme::StyledGraphemes, pane::Pane,
};

mod history;
pub use history::History;
mod text_editor;
pub use text_editor::{Mode, TextEditor};

#[derive(Clone, Default)]
pub struct State {
    /// The `TextEditor` component to be rendered.
    pub texteditor: TextEditor,
    /// Optional history for navigating through previous inputs.
    pub history: Option<History>,

    /// Prompt string displayed before the input text.
    pub prefix: String,
    /// Optional character used for masking the input string (e.g., for password fields).
    pub mask: Option<char>,

    /// Style applied to the prompt string.
    pub prefix_style: ContentStyle,
    /// Style applied to the currently selected character.
    pub active_char_style: ContentStyle,
    /// Style applied to characters that are not currently selected.
    pub inactive_char_style: ContentStyle,

    /// Current edit mode, determining whether input inserts or overwrites existing text.
    pub edit_mode: Mode,
    /// Characters to be for word break.
    pub word_break_chars: HashSet<char>,
    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let mut buf = StyledGraphemes::default();

        let mut styled_prefix = StyledGraphemes::from_str(&self.prefix, self.prefix_style);

        buf.append(&mut styled_prefix);

        let text = match self.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = text
            .apply_style(self.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.active_char_style);

        buf.append(&mut styled);

        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let (matrix, offset) = buf.matrixify(
            width as usize,
            height,
            (StyledGraphemes::from_str(&self.prefix, self.prefix_style).widths()
                + self.texteditor.position())
                / width as usize,
        );

        Pane::new(matrix, offset)
    }
}
