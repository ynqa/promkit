use std::collections::HashSet;

use promkit_core::{Pane, PaneFactory, grapheme::StyledGraphemes};

mod history;
pub use history::History;
#[path = "text_editor/text_editor.rs"]
mod inner;
pub use inner::{Mode, TextEditor};
pub mod format;
use format::Formatter;

#[derive(Clone, Default)]
pub struct State {
    /// The `TextEditor` component to be rendered.
    pub texteditor: TextEditor,
    /// Optional history for navigating through previous inputs.
    pub history: Option<History>,

    /// Rendering options for this widget.
    pub formatter: Formatter,

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

        let mut styled_prefix =
            StyledGraphemes::from_str(&self.formatter.prefix, self.formatter.prefix_style);

        buf.append(&mut styled_prefix);

        let text = match self.formatter.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = text
            .apply_style(self.formatter.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.formatter.active_char_style);

        buf.append(&mut styled);

        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let (matrix, offset) = buf.matrixify(
            width as usize,
            height,
            (StyledGraphemes::from_str(&self.formatter.prefix, self.formatter.prefix_style)
                .widths()
                + self.texteditor.position())
                / width as usize,
        );

        Pane::new(matrix, offset)
    }
}
