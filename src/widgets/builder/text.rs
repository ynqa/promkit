use anyhow::Result;

use crate::{
    crossterm::style::ContentStyle,
    widgets::{text::Text, State},
};

pub struct TextBuilder {
    text: String,
    style: ContentStyle,
}

impl TextBuilder {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            style: ContentStyle::new(),
        }
    }

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

    pub fn build(self) -> Result<Box<State<Text>>> {
        Ok(Box::new(State::<Text>::new(Text {
            text: self.text,
            style: self.style,
        })))
    }
}
