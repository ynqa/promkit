use anyhow::Result;

use crate::{editor::text::Text, grapheme::Graphemes};

pub struct TextBuilder {
    text: Graphemes,
}

impl TextBuilder {
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            text: Graphemes::from(text),
        }
    }

    pub fn build(self) -> Result<Box<Text>> {
        Ok(Box::new(Text { text: self.text }))
    }
}
