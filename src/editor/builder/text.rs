use anyhow::Result;

use crate::{crossterm::style::ContentStyle, editor::text::Text, grapheme::Graphemes};

pub struct TextBuilder {
    text: String,
    style: ContentStyle,
}

impl TextBuilder {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text: text.as_ref().to_string(),
            style: ContentStyle::new(),
        }
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Result<Box<Text>> {
        Ok(Box::new(Text {
            text: Graphemes::new_with_style(self.text, self.style),
        }))
    }
}
