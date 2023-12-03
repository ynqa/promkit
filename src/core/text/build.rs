use crate::{crossterm::style::ContentStyle, error::Result, render::State};

use super::Renderer;

#[derive(Clone, Default)]
pub struct Builder {
    text: String,
    style: ContentStyle,
}

impl Builder {
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

    pub fn build(self) -> Result<Renderer> {
        Ok(Renderer {
            text: self.text,
            style: self.style,
        })
    }

    pub fn build_state(self) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(self.build()?)))
    }
}
