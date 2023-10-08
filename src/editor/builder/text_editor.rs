use anyhow::Result;

use crate::{
    crossterm::style::ContentStyle, editor::text_editor::TextEditor, grapheme::Graphemes,
    text_buffer::TextBuffer,
};

pub struct TextEditorBuilder {
    label: String,
    label_style: ContentStyle,
}

impl TextEditorBuilder {
    pub fn new() -> Self {
        Self {
            label: String::from("❯❯ "),
            label_style: ContentStyle::new(),
        }
    }

    pub fn label<T: AsRef<str>>(mut self, label: T) -> Self {
        self.label = label.as_ref().to_string();
        self
    }

    pub fn label_style(mut self, style: ContentStyle) -> Self {
        self.label_style = style;
        self
    }

    pub fn build(self) -> Result<Box<TextEditor>> {
        Ok(Box::new(TextEditor {
            textbuffer: TextBuffer::new(),
            label: Graphemes::new_with_style(self.label, self.label_style),
        }))
    }
}
