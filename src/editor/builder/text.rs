use anyhow::Result;

use crate::{editor::text::Text, grapheme::Graphemes, crossterm::style::ContentStyle};

pub struct TextBuilder {
    text: Graphemes,
    style: ContentStyle,
}

impl TextBuilder {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text: Graphemes::from(text),
            style: ContentStyle::new(),
        }
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Result<Box<Text>> {
        Ok(Box::new(Text { text: self.text }))
    }
}
