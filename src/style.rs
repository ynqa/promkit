use crate::crossterm::style::{Attributes, Color, ContentStyle};

#[derive(Default)]
pub struct Style {
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    underline_color: Option<Color>,
    attributes: Attributes,
}

impl Style {
    pub fn new() -> Self {
        Style::default()
    }

    pub fn fgc(mut self, color: Color) -> Self {
        self.foreground_color = Some(color);
        self
    }

    pub fn bgc(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn ulc(mut self, color: Color) -> Self {
        self.underline_color = Some(color);
        self
    }

    pub fn attrs(mut self, attributes: Attributes) -> Self {
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
