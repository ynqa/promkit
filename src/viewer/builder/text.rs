use crate::{crossterm::style::ContentStyle, error::Result};

use super::super::{text::Text, State};

#[derive(Clone, Default)]
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

    pub fn text<T: AsRef<str>>(mut self, text: T) -> Self {
        self.text = text.as_ref().to_string();
        self
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Result<Text> {
        Ok(Text {
            text: self.text,
            style: self.style,
        })
    }

    pub fn build_state(self) -> Result<Box<State<Text>>> {
        Ok(Box::new(State::<Text>::new(self.build()?)))
    }
}
