use std::{cell::RefCell, collections::HashSet, io};

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    listbox::{self, Listbox},
    snapshot::Snapshot,
    style::StyleBuilder,
    suggest::Suggest,
    switch::ActiveKeySwitcher,
    text,
    text_editor::{self, History},
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    Prompt,
};

pub mod confirm;
pub mod keymap;
pub mod password;
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
    /// Writer to which promptkit write its contents
    writer: Box<dyn io::Write>,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default as keymap::Keymap)
                .register("on_suggest", self::keymap::on_suggest),
            title_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_state: text_editor::State {
                texteditor: Default::default(),
                history: Default::default(),
                prefix: String::from("❯❯ "),
                mask: Default::default(),
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                word_break_chars: HashSet::from([' ']),
                lines: Default::default(),
            },
            suggest: Default::default(),
            suggest_state: listbox::State {
                listbox: Listbox::from_displayable(Vec::<String>::new()),
                cursor: String::from("❯ "),
                active_item_style: Some(
                    StyleBuilder::new()
                        .fgc(Color::DarkGrey)
                        .bgc(Color::DarkYellow)
                        .build(),
                ),
                inactive_item_style: Some(StyleBuilder::new().fgc(Color::DarkGrey).build()),
                lines: Some(3),
            },
            validator: Default::default(),
            error_message_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .fgc(Color::DarkRed)
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            writer: Box::new(io::stdout()),
        }
    }
}

impl Readline {
    /// Sets the title text displayed above the input field.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = text.as_ref().to_string();
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

    /// Sets writer.
    pub fn writer<W: io::Write + 'static>(mut self, writer: W) -> Self {
        self.writer = Box::new(writer);
        self
    }

    /// Initiates the prompt process,
    /// displaying the configured UI elements and handling user input.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_snapshot: Snapshot::<text::State>::new(self.title_state),
                text_editor_snapshot: Snapshot::<text_editor::State>::new(self.text_editor_state),
                suggest: self.suggest,
                suggest_snapshot: Snapshot::<listbox::State>::new(self.suggest_state),
                validator: self.validator,
                error_message_snapshot: Snapshot::<text::State>::new(self.error_message_state),
            },
            writer: self.writer,
        })
    }
}
