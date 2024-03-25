use std::collections::HashSet;

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    listbox::{self, Listbox},
    snapshot::Snapshot,
    style::StyleBuilder,
    suggest::Suggest,
    text,
    text_editor::{self, History},
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    EventHandler, Prompt, PromptSignal, Renderer,
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
    keymap: KeymapManager<self::render::Renderer>,
    /// Renderer for the title displayed above the input field.
    title_renderer: text::Renderer,
    /// Renderer for the text editor where user input is entered.
    text_editor_renderer: text_editor::Renderer,
    suggest: Option<Suggest>,
    suggest_renderer: listbox::Renderer,
    /// Optional validator for input validation with custom error messages.
    validator: Option<ValidatorManager<str>>,
    /// Renderer for displaying error messages based on input validation.
    error_message_renderer: text::Renderer,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            keymap: KeymapManager::new("default", self::keymap::default)
                .register("on_suggest", self::keymap::on_suggest),
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
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
            suggest_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(Vec::<String>::new()),
                cursor: String::from("❯ "),
                active_item_style: StyleBuilder::new()
                    .fgc(Color::DarkGrey)
                    .bgc(Color::DarkYellow)
                    .build(),
                inactive_item_style: StyleBuilder::new().fgc(Color::DarkGrey).build(),
                lines: Some(3),
            },
            validator: Default::default(),
            error_message_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .fgc(Color::DarkRed)
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
        }
    }
}

impl Readline {
    /// Sets the title text displayed above the input field.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Enables suggestion functionality with the provided `Suggest` instance.
    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = Some(suggest);
        self
    }

    /// Enables history functionality allowing navigation through previous inputs.
    pub fn enable_history(mut self) -> Self {
        self.text_editor_renderer.history = Some(History::default());
        self
    }

    /// Sets the prefix string displayed before the input text.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_renderer.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the character used for masking input text, typically used for password fields.
    pub fn mask(mut self, mask: char) -> Self {
        self.text_editor_renderer.mask = Some(mask);
        self
    }

    /// Sets the style for the prefix string.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.prefix_style = style;
        self
    }

    /// Sets the style for the currently active character in the input field.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.active_char_style = style;
        self
    }

    /// Sets the style for characters that are not currently active in the input field.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.inactive_char_style = style;
        self
    }

    /// Sets the edit mode for the text editor, either insert or overwrite.
    pub fn edit_mode(mut self, mode: text_editor::Mode) -> Self {
        self.text_editor_renderer.edit_mode = mode;
        self
    }

    /// Sets the characters to be for word break.
    pub fn word_break_chars(mut self, characters: HashSet<char>) -> Self {
        self.text_editor_renderer.word_break_chars = characters;
        self
    }

    /// Sets the number of lines available for rendering the text editor.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_renderer.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(
        mut self,
        key: K,
        handler: EventHandler<self::render::Renderer>,
    ) -> Self {
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
    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            Box::new(self::render::Renderer {
                keymap: self.keymap,
                title_snapshot: Snapshot::<text::Renderer>::new(self.title_renderer),
                text_editor_snapshot: Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                ),
                suggest: self.suggest,
                suggest_snapshot: Snapshot::<listbox::Renderer>::new(self.suggest_renderer),
                validator: self.validator,
                error_message_snapshot: Snapshot::<text::Renderer>::new(
                    self.error_message_renderer,
                ),
            }),
            Box::new(
                move |event: &Event,
                      renderer: &mut Box<dyn Renderer + 'static>|
                      -> Result<PromptSignal> {
                    let renderer = self::render::Renderer::cast_mut(renderer.as_mut())?;
                    match renderer.keymap.get() {
                        Some(f) => f(event, renderer),
                        None => Ok(PromptSignal::Quit),
                    }
                },
            ),
            |renderer: &(dyn Renderer + '_)| -> Result<String> {
                Ok(self::render::Renderer::cast(renderer)?
                    .text_editor_snapshot
                    .after()
                    .texteditor
                    .text_without_cursor()
                    .to_string())
            },
        )
    }
}
