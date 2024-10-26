use crate::{
    crossterm::style::ContentStyle,
    validate::{ErrorMessageGenerator, Validator},
    Prompt,
};

use super::{render, Readline};

/// A specialized `Readline` struct for securely capturing password input.
/// It masks the input with a specified character for privacy and security.
pub struct Password(Readline);

impl Default for Password {
    /// Creates a new `Password` instance with default settings, using '*' as the mask character.
    fn default() -> Self {
        Self(Readline::default().mask('*'))
    }
}

impl Password {
    /// Sets the title text displayed above the password input field.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self = Password(self.0.title(text));
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.title_style(style));
        self
    }

    /// Sets the character used for masking the password input.
    pub fn mask(mut self, mask: char) -> Self {
        self = Password(self.0.mask(mask));
        self
    }

    /// Sets the style for the currently active character in the password input field.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.active_char_style(style));
        self
    }

    /// Sets the style for characters that are not currently active in the password input field.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.inactive_char_style(style));
        self
    }

    /// Sets the number of lines available for rendering the password input field.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self = Password(self.0.text_editor_lines(lines));
        self
    }

    /// Configures a validator for the password input with a function to validate the input and another to configure the error message.
    pub fn validator(
        mut self,
        validator: Validator<str>,
        error_message_generator: ErrorMessageGenerator<str>,
    ) -> Self {
        self = Password(self.0.validator(validator, error_message_generator));
        self
    }

    /// Displays the password prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the user's input.
    pub fn prompt<W: std::io::Write>(
        self,
        writer: W,
    ) -> anyhow::Result<Prompt<render::Renderer, W>> {
        self.0.prompt(writer)
    }
}
