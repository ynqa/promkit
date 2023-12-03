use crate::{
    crossterm::style::ContentStyle,
    error::Result,
    text::Text,
    view::{History, Suggest},
};

use super::super::{text_editor::TextEditorViewer, Mode, State};

#[derive(Clone, Default)]
pub struct TextEditorViewerBuilder {
    history: Option<History>,
    suggest: Suggest,
    prefix: String,
    prefix_style: ContentStyle,
    style: ContentStyle,
    cursor_style: ContentStyle,
    mode: Mode,
    mask: Option<char>,
    lines: Option<usize>,
}

impl TextEditorViewerBuilder {
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.prefix = prefix.as_ref().to_string();
        self
    }

    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.prefix_style = style;
        self
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: ContentStyle) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = Some(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = suggest;
        self
    }

    pub fn enable_history(mut self) -> Self {
        self.history = Some(History::default());
        self
    }

    pub fn build(self) -> Result<TextEditorViewer> {
        Ok(TextEditorViewer {
            text: Text::default(),
            history: self.history,
            suggest: self.suggest,
            prefix: self.prefix,
            prefix_style: self.prefix_style,
            style: self.style,
            cursor_style: self.cursor_style,
            mode: self.mode,
            mask: self.mask,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<TextEditorViewer>>> {
        Ok(Box::new(State::<TextEditorViewer>::new(self.build()?)))
    }
}
