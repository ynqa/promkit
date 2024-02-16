use crate::{crossterm::style::ContentStyle, error::Result, Prompt};

use super::Readline;

pub struct Password(Readline);

impl Default for Password {
    fn default() -> Self {
        Self(Readline::default().mask('*'))
    }
}

impl Password {
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self = Password(self.0.title(text));
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.title_style(style));
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self = Password(self.0.mask(mask));
        self
    }

    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.active_char_style(style));
        self
    }

    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self = Password(self.0.inactive_char_style(style));
        self
    }

    pub fn window_size(mut self, window_size: usize) -> Self {
        self = Password(self.0.window_size(window_size));
        self
    }

    pub fn validator<V, F>(mut self, validator: V, error_message_configure: F) -> Self
    where
        V: Fn(&str) -> bool + 'static,
        F: Fn(&str) -> String + 'static,
    {
        self = Password(self.0.validator(validator, error_message_configure));
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        self.0.prompt()
    }
}
