use std::collections::HashSet;

use crate::{
    crossterm::style::ContentStyle,
    grapheme::{matrixify, StyledGraphemes},
    impl_as_any,
    pane::Pane,
    PaneFactory,
};

use super::{History, Mode, TextEditor};

#[derive(Clone)]
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

impl_as_any!(State);

impl PaneFactory for State {
    fn create_pane(&self, width: u16) -> Pane {
        let mut buf = StyledGraphemes::default();
        buf.append(&mut StyledGraphemes::from_str(
            &self.prefix,
            self.prefix_style,
        ));

        let text = match self.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = StyledGraphemes::from_graphemes(text, self.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.active_char_style);

        buf.append(&mut styled);

        Pane::new(
            matrixify(width as usize, &buf),
            self.texteditor.position() / width as usize,
            self.lines,
        )
    }
}
