use anyhow::Result;

use crate::{
    crossterm::style::ContentStyle, editor::text_editor::TextEditor, grapheme::Graphemes,
    text_buffer::TextBuffer,
};

pub struct TextEditorBuilder {
    label: String,
    style: ContentStyle,
    cursor_style: ContentStyle,
    label_style: ContentStyle,
}

impl Default for TextEditorBuilder {
    fn default() -> Self {
        Self {
            label: String::from("❯❯ "),
            style: ContentStyle::new(),
            cursor_style: ContentStyle::new(),
            label_style: ContentStyle::new(),
        }
    }
}

impl TextEditorBuilder {
    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: ContentStyle) -> Self {
        self.cursor_style = style;
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

    pub fn build(self) -> Result<Box<TextEditor>> {
        Ok(Box::new(TextEditor {
            textbuffer: TextBuffer::default(),
            style: self.style,
            cursor_style: self.cursor_style,
            label: Graphemes::new_with_style(self.label, self.label_style),
        }))
    }
}
