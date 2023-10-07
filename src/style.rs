use crate::crossterm::style::{Attributes, Color, ContentStyle};

#[derive(Default)]
pub struct ContentStyleBuilder {
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    underline_color: Option<Color>,
    attributes: Attributes,
}

impl ContentStyleBuilder {
    pub fn new() -> Self {
        ContentStyleBuilder::default()
    }

    pub fn foreground_color(mut self, color: Color) -> Self {
        self.foreground_color = Some(color);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn underline_color(mut self, color: Color) -> Self {
        self.underline_color = Some(color);
        self
    }

    pub fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn build(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: self.foreground_color,
            background_color: self.background_color,
            underline_color: self.underline_color,
            attributes: self.attributes,
        }
    }
}
