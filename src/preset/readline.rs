use std::cell::RefCell;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
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
    EventAction, EventHandler, Prompt, Renderer,
};

mod confirm;
pub use confirm::Confirm;
mod password;
pub use password::Password;
mod suggest;

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
    suggest: Option<RefCell<Suggest>>,
    suggest_renderer: listbox::Renderer,
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
            suggest: Default::default(),
            suggest_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(Vec::<String>::new()),
                keymap: KeymapManager::new("default", |_, _| Ok(EventAction::Continue)),
                cursor: String::from("❯ "),
                active_item_style: StyleBuilder::new()
                    .fgc(Color::DarkGrey)
                    .bgc(Color::DarkYellow)
                    .build(),
                inactive_item_style: StyleBuilder::new().fgc(Color::DarkGrey).build(),
                lines: Some(3),
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
        self.text_editor_renderer.keymap = self
            .text_editor_renderer
            .keymap
            .register("default", suggest::default_text_editor_keymap)
            .register("on_suggest", suggest::on_suggest_text_editor_keymap);
        self.suggest_renderer.keymap = self
            .suggest_renderer
            .keymap
            .register("default", suggest::default_suggest_keymap)
            .register("on_suggest", suggest::on_suggest_suggest_keymap);
        self.suggest = Some(RefCell::new(suggest));
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
        let suggest = self.suggest;

        Prompt::try_new(
            vec![
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(Snapshot::<text::Renderer>::new(self.error_message_renderer)),
                Box::new(Snapshot::<listbox::Renderer>::new(self.suggest_renderer)),
            ],
            move |event: &Event, renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<bool> {
                let text_editor_snapshot =
                    Snapshot::<text_editor::Renderer>::cast(renderers[1].as_ref())?;
                let error_message_snapshot =
                    Snapshot::<text::Renderer>::cast(renderers[2].as_ref())?;
                let suggest_snapshot = Snapshot::<listbox::Renderer>::cast(renderers[3].as_ref())?;

                let mut text_editor_borrowed_after = text_editor_snapshot.borrow_mut_after();
                let mut suggest_borrowed_after = suggest_snapshot.borrow_mut_after();

                let text = text_editor_borrowed_after
                    .texteditor
                    .text_without_cursor()
                    .to_string();
                let active_key = text_editor_borrowed_after.keymap.active_key();

                if let Some(suggest) = suggest.as_ref() {
                    match active_key {
                        "default" => {
                            if let Event::Key(KeyEvent {
                                code: KeyCode::Tab,
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            }) = event
                            {
                                if let Some(candidates) = suggest.borrow().prefix_search(&text) {
                                    suggest_borrowed_after.listbox = Listbox::from_iter(candidates);
                                    text_editor_borrowed_after
                                        .texteditor
                                        .replace(&suggest_borrowed_after.listbox.get());

                                    text_editor_borrowed_after.keymap.switch("on_suggest");
                                    suggest_borrowed_after.keymap.switch("on_suggest");
                                }
                            }
                        }
                        "on_suggest" => match event {
                            Event::Key(KeyEvent {
                                code: KeyCode::Tab,
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            })
                            | Event::Key(KeyEvent {
                                code: KeyCode::Down,
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            })
                            | Event::Key(KeyEvent {
                                code: KeyCode::Tab,
                                modifiers: KeyModifiers::SHIFT,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            })
                            | Event::Key(KeyEvent {
                                code: KeyCode::Up,
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            }) => {
                                text_editor_borrowed_after
                                    .texteditor
                                    .replace(&suggest_borrowed_after.listbox.get());
                            }
                            _ => {
                                suggest_borrowed_after.listbox =
                                    Listbox::from_iter(Vec::<String>::new());

                                text_editor_borrowed_after.keymap.switch("default");
                                suggest_borrowed_after.keymap.switch("default");
                            }
                        },
                        _ => (),
                    }
                }

                Ok(
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) = event
                    {
                        validator
                            .as_ref()
                            .map(|validator| {
                                let valid = validator.validate(&text);
                                if !valid {
                                    error_message_snapshot.borrow_mut_after().text =
                                        validator.generate_error_message(&text);
                                }
                                valid
                            })
                            .unwrap_or(true)
                    } else {
                        true
                    },
                )
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
