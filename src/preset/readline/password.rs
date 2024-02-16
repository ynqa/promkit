use crate::{error::Result, Prompt};

use super::{Readline, Theme};

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

    pub fn theme(mut self, theme: Theme) -> Self {
        self = Password(self.0.theme(theme));
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self = Password(self.0.mask(mask));
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
