use anyhow::Result;

use crate::{editor::text_editor::TextEditor, grapheme::Graphemes, text_buffer::TextBuffer};

pub struct TextEditorBuilder {
    label: Graphemes,
}

impl TextEditorBuilder {
    pub fn new() -> Self {
        Self {
            label: Graphemes::from("❯❯ "),
        }
    }

    pub fn label<T: AsRef<str>>(mut self, label: T) -> Self {
        self.label = Graphemes::from(label);
        self
    }

    pub fn build(self) -> Result<Box<TextEditor>> {
        Ok(Box::new(TextEditor {
            textbuffer: TextBuffer::new(),
            label: self.label,
        }))
    }
}
