use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{matrixify, StyledGraphemes},
    keymap::KeymapManager,
    pane::Pane,
    AsAny, EventAction, Result,
};

use super::{History, Mode, TextEditor};

/// Represents a renderer for the `TextEditor` component,
/// capable of visualizing text input in a pane.
/// It supports a variety of features including history navigation,
/// input suggestions, input masking,
/// customizable prompt strings,
/// and styles for different parts of the input. It also handles different
/// edit modes such as insert and overwrite,
/// and can be configured to render a specific number of lines.
#[derive(Clone)]
pub struct Renderer {
    /// The `TextEditor` component to be rendered.
    pub texteditor: TextEditor,
    /// Optional history for navigating through previous inputs.
    pub history: Option<History>,

    pub keymap: KeymapManager<Self>,

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
    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl crate::Renderer for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
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

    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        (self.keymap.get())(self, event)
    }

    fn postrun(&mut self) {
        if let Some(ref mut history) = &mut self.history {
            history.insert(self.texteditor.text_without_cursor().to_string());
        }
        self.texteditor = TextEditor::default();
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
