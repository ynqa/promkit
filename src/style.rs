use crate::crossterm::style::{Attributes, Color, ContentStyle};

/// A struct for defining and building styles for terminal text.
///
/// This struct allows for the customization of text appearance in the terminal,
/// including foreground, background, and underline colors, as well as text attributes
/// like bold, italic, etc. It provides a fluent interface for setting these properties,
/// and a method to build a `ContentStyle` that can be applied to text.
#[derive(Default)]
pub struct Style {
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    underline_color: Option<Color>,
    attributes: Attributes,
}

impl Style {
    /// Creates a new `Style` instance with default values.
    pub fn new() -> Self {
        Style::default()
    }

    /// Sets the foreground color of the style.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to set as the foreground color.
    ///
    /// # Returns
    ///
    /// Returns the `Style` instance to allow for method chaining.
    pub fn fgc(mut self, color: Color) -> Self {
        self.foreground_color = Some(color);
        self
    }

    /// Sets the background color of the style.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to set as the background color.
    ///
    /// # Returns
    ///
    /// Returns the `Style` instance to allow for method chaining.
    pub fn bgc(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Sets the underline color of the style.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to set as the underline color.
    ///
    /// # Returns
    ///
    /// Returns the `Style` instance to allow for method chaining.
    pub fn ulc(mut self, color: Color) -> Self {
        self.underline_color = Some(color);
        self
    }

    /// Sets the attributes of the style.
    ///
    /// # Arguments
    ///
    /// * `attributes` - The attributes to set.
    ///
    /// # Returns
    ///
    /// Returns the `Style` instance to allow for method chaining.
    pub fn attrs(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }

    /// Builds and returns a `ContentStyle` based on the set properties.
    ///
    /// # Returns
    ///
    /// Returns a `ContentStyle` that can be applied to text.
    pub fn build(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: self.foreground_color,
            background_color: self.background_color,
            underline_color: self.underline_color,
            attributes: self.attributes,
        }
    }
}
