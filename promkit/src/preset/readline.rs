//! Offers functionality for reading input from the user.

use std::{cell::RefCell, collections::HashSet};

use promkit_widgets::{
    listbox::{self, Listbox},
    text::{self, Text},
    text_editor::{self, History},
};

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    snapshot::Snapshot,
    suggest::Suggest,
    switch::ActiveKeySwitcher,
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    Prompt,
};

pub mod keymap;
pub mod render;

/// `Readline` struct provides functionality
/// for reading a single line of input from the user.
/// It supports various configurations
/// such as input masking, history, suggestions, and custom styles.
pub struct Readline {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// State for the title displayed above the input field.
    title_state: text::State,
    /// State for the text editor where user input is entered.
    text_editor_state: text_editor::State,
    suggest: Option<Suggest>,
    suggest_state: listbox::State,
    /// Optional validator for input validation with custom error messages.
    validator: Option<ValidatorManager<str>>,
    /// State for displaying error messages based on input validation.
    error_message_state: text::State,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default as keymap::Keymap)
                .register("on_suggest", self::keymap::on_suggest),
            title_state: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            text_editor_state: text_editor::State {
                texteditor: Default::default(),
                history: Default::default(),
                prefix: String::from("❯❯ "),
                mask: Default::default(),
                prefix_style: ContentStyle {
                    foreground_color: Some(Color::DarkGreen),
                    ..Default::default()
                },

                active_char_style: ContentStyle {
                    background_color: Some(Color::DarkCyan),
                    ..Default::default()
                },
                inactive_char_style: ContentStyle::default(),
                edit_mode: Default::default(),
                word_break_chars: HashSet::from([' ']),
                lines: Default::default(),
            },
            suggest: Default::default(),
            suggest_state: listbox::State {
                listbox: Listbox::from_displayable(Vec::<String>::new()),
                cursor: String::from("❯ "),
                active_item_style: Some(ContentStyle {
                    foreground_color: Some(Color::DarkGrey),
                    background_color: Some(Color::DarkYellow),
                    ..Default::default()
                }),
                inactive_item_style: Some(ContentStyle {
                    foreground_color: Some(Color::DarkGrey),
                    ..Default::default()
                }),
                lines: Some(3),
            },
            validator: Default::default(),
            error_message_state: text::State {
                text: Default::default(),
                style: ContentStyle {
                    foreground_color: Some(Color::DarkRed),
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                lines: None,
            },
        }
    }
}

impl Readline {
    /// Sets the title text displayed above the input field.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_state.style = style;
        self
    }

    /// Enables suggestion functionality with the provided `Suggest` instance.
    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = Some(suggest);
        self
    }

    /// Enables history functionality allowing navigation through previous inputs.
    pub fn enable_history(mut self) -> Self {
        self.text_editor_state.history = Some(History::default());
        self
    }

    /// Sets the prefix string displayed before the input text.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_state.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the character used for masking input text, typically used for password fields.
    pub fn mask(mut self, mask: char) -> Self {
        self.text_editor_state.mask = Some(mask);
        self
    }

    /// Sets the style for the prefix string.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.prefix_style = style;
        self
    }

    /// Sets the style for the currently active character in the input field.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.active_char_style = style;
        self
    }

    /// Sets the style for characters that are not currently active in the input field.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.inactive_char_style = style;
        self
    }

    /// Sets the edit mode for the text editor, either insert or overwrite.
    pub fn edit_mode(mut self, mode: text_editor::Mode) -> Self {
        self.text_editor_state.edit_mode = mode;
        self
    }

    /// Sets the characters to be for word break.
    pub fn word_break_chars(mut self, characters: HashSet<char>) -> Self {
        self.text_editor_state.word_break_chars = characters;
        self
    }

    /// Sets the number of lines available for rendering the text editor.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_state.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(mut self, key: K, handler: keymap::Keymap) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Configures a validator for the input with a function to validate the input and another to configure the error message.
    pub fn validator(
        mut self,
        validator: Validator<str>,
        error_message_generator: ErrorMessageGenerator<str>,
    ) -> Self {
        self.validator = Some(ValidatorManager::new(validator, error_message_generator));
        self
    }

    /// Initiates the prompt process,
    /// displaying the configured UI elements and handling user input.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_state: self.title_state,
                text_editor_snapshot: Snapshot::<text_editor::State>::new(self.text_editor_state),
                suggest: self.suggest,
                suggest_snapshot: Snapshot::<listbox::State>::new(self.suggest_state),
                validator: self.validator,
                error_message_snapshot: Snapshot::<text::State>::new(self.error_message_state),
            },
        })
    }
}
