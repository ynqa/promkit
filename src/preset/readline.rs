use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    snapshot::Snapshot,
    style::StyleBuilder,
    text,
    text_editor::{self, History, Suggest},
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    EventHandler, Prompt, Renderer,
};

mod confirm;
pub use confirm::Confirm;
mod password;
pub use password::Password;

/// `Readline` struct provides functionality
/// for reading a single line of input from the user.
/// It supports various configurations
/// such as input masking, history, suggestions, and custom styles.
pub struct Readline {
    /// Renderer for the title displayed above the input field.
    title_renderer: text::Renderer,
    /// Renderer for the text editor where user input is entered.
    text_editor_renderer: text_editor::Renderer,
    /// Renderer for displaying error messages based on input validation.
    error_message_renderer: text::Renderer,
    /// Optional validator for input validation with custom error messages.
    validator: Option<ValidatorManager<str>>,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: Default::default(),
                suggest: Default::default(),
                keymap: KeymapManager::new("default", text_editor::keymap::default_keymap),
                prefix: String::from("❯❯ "),
                mask: Default::default(),
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                lines: Default::default(),
            },
            error_message_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .fgc(Color::DarkRed)
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            validator: Default::default(),
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
        self.text_editor_renderer.suggest = suggest;
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

    /// Sets the number of lines available for rendering the text editor.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_renderer.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(
        mut self,
        key: K,
        handler: EventHandler<text_editor::Renderer>,
    ) -> Self {
        self.text_editor_renderer.keymap = self.text_editor_renderer.keymap.register(key, handler);
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
        let validator = self.validator;

        Prompt::try_new(
            vec![
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(Snapshot::<text::Renderer>::new(self.error_message_renderer)),
            ],
            move |event: &Event, renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<bool> {
                let text = Snapshot::<text_editor::Renderer>::cast_and_borrow_after(
                    renderers[1].as_ref(),
                )?
                .texteditor
                .text_without_cursor()
                .to_string();

                let error_message = Snapshot::<text::Renderer>::cast(renderers[2].as_ref())?;
                let ret = match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => match &validator {
                        Some(validator) => {
                            let ret = validator.validate(&text);
                            if !validator.validate(&text) {
                                error_message.borrow_mut_after().text =
                                    validator.generate_error_message(&text);
                            }
                            ret
                        }
                        None => true,
                    },
                    _ => true,
                };
                if ret {
                    error_message.reset_after_to_init()
                }
                Ok(ret)
            },
            |renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<String> {
                Ok(
                    Snapshot::<text_editor::Renderer>::cast_and_borrow_after(
                        renderers[1].as_ref(),
                    )?
                    .texteditor
                    .text_without_cursor()
                    .to_string(),
                )
            },
            false,
        )
    }
}
