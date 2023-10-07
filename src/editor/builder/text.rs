use anyhow::Result;

use crate::{crossterm::style::ContentStyle, editor::text::Text, grapheme::Graphemes};

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
