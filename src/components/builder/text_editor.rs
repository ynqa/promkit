use anyhow::Result;

use crate::{
    crossterm::style::ContentStyle, history::History, suggest::Suggest, text_buffer::TextBuffer,
};

use super::super::{text_editor::TextEditor, Mode, State};

pub struct TextEditorBuilder {
    history: Option<History>,
    suggest: Suggest,
    label: String,
    label_style: ContentStyle,
    style: ContentStyle,
    cursor_style: ContentStyle,
    mode: Mode,
    mask: Option<char>,
    lines: Option<usize>,
}

impl Default for TextEditorBuilder {
    fn default() -> Self {
        Self {
            history: Some(History::default()),
            suggest: Suggest::default(),
            label: String::from("❯❯ "),
            label_style: ContentStyle::new(),
            style: ContentStyle::new(),
            cursor_style: ContentStyle::new(),
            mode: Mode::Insert,
            mask: None,
            lines: None,
        }
    }
}

impl TextEditorBuilder {
    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = suggest;
        self
    }

    pub fn label<T: AsRef<str>>(mut self, label: T) -> Self {
        self.label = label.as_ref().to_string();
        self
    }

    pub fn label_style(mut self, style: ContentStyle) -> Self {
        self.label_style = style;
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

    pub fn mode(mut self, mode: Mode) -> Self {
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

    pub fn disable_history(mut self) -> Self {
        self.history = None;
        self
    }

    pub fn build(self) -> Result<TextEditor> {
        Ok(TextEditor {
            textbuffer: TextBuffer::default(),
            history: self.history,
            suggest: self.suggest,
            label: self.label,
            label_style: self.label_style,
            style: self.style,
            cursor_style: self.cursor_style,
            mode: self.mode,
            mask: self.mask,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<TextEditor>>> {
        Ok(Box::new(State::<TextEditor>::new(self.build()?)))
    }
}
