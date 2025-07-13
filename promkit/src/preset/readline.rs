//! Offers functionality for reading input from the user.

use std::collections::HashSet;

use crate::{
    core::{
        crossterm::{
            self,
            event::Event,
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        PaneFactory,
    },
    preset::Evaluator,
    suggest::Suggest,
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    widgets::{
        listbox::{self, Listbox},
        text::{self, Text},
        text_editor::{self, History},
    },
    Signal,
};

pub mod evaluate;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
/// Represent the indices of various components in the readline preset.
pub enum Index {
    Title = 0,
    Readline = 1,
    Suggestion = 2,
    ErrorMessage = 3,
}

/// Represents the focus state of the readline,
/// determining which component is currently active for input handling.
pub enum Focus {
    Readline,
    Suggestion,
}

/// `Readline` struct provides functionality
/// for reading a single line of input from the user.
/// It supports various configurations
/// such as input masking, history, suggestions, and custom styles.
pub struct Readline {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// Holds the focus state for event handling, determining which component is currently focused.
    pub focus: Focus,
    /// Holds a title's renderer state, used for rendering the title section.
    pub title: text::State,
    /// Holds a text editor's renderer state, used for rendering the text input area.
    pub readline: text_editor::State,
    /// Optional suggest component for autocomplete functionality.
    pub suggest: Option<Suggest>,
    /// Holds a suggest box's renderer state, used when rendering suggestions for autocomplete.
    pub suggestions: listbox::State,
    /// Optional validator manager for input validation.
    pub validator: Option<ValidatorManager<str>>,
    /// Holds an error message's renderer state, used for rendering error messages.
    pub error_message: text::State,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            focus: Focus::Readline,
            title: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            readline: text_editor::State {
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
            suggestions: listbox::State {
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
            error_message: text::State {
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

#[async_trait::async_trait]
impl crate::Prompt for Readline {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Readline, self.readline.create_pane(size.0, size.1)),
                    (
                        Index::Suggestion,
                        self.suggestions.create_pane(size.0, size.1),
                    ),
                    (
                        Index::ErrorMessage,
                        self.error_message.create_pane(size.0, size.1),
                    ),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator)(event, self).await;
        let size = crossterm::terminal::size()?;
        self.renderer
            .as_ref()
            .unwrap()
            .update([
                (Index::Title, self.title.create_pane(size.0, size.1)),
                (Index::Readline, self.readline.create_pane(size.0, size.1)),
                (
                    Index::Suggestion,
                    self.suggestions.create_pane(size.0, size.1),
                ),
                (
                    Index::ErrorMessage,
                    self.error_message.create_pane(size.0, size.1),
                ),
            ])
            .render()
            .await?;
        ret
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        let ret = self.readline.texteditor.text_without_cursor().to_string();

        // Reset the text editor state for the next prompt.
        self.readline.texteditor.erase_all();

        Ok(ret)
    }
}

impl Readline {
    /// Sets the title text displayed above the input field.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.style = style;
        self
    }

    /// Enables suggestion functionality with the provided `Suggest` instance.
    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = Some(suggest);
        self
    }

    /// Enables history functionality allowing navigation through previous inputs.
    pub fn enable_history(mut self) -> Self {
        self.readline.history = Some(History::default());
        self
    }

    /// Sets the prefix string displayed before the input text.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.readline.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the character used for masking input text, typically used for password fields.
    pub fn mask(mut self, mask: char) -> Self {
        self.readline.mask = Some(mask);
        self
    }

    /// Sets the style for the prefix string.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.readline.prefix_style = style;
        self
    }

    /// Sets the style for the currently active character in the input field.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.active_char_style = style;
        self
    }

    /// Sets the style for characters that are not currently active in the input field.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.inactive_char_style = style;
        self
    }

    /// Sets the edit mode for the text editor, either insert or overwrite.
    pub fn edit_mode(mut self, mode: text_editor::Mode) -> Self {
        self.readline.edit_mode = mode;
        self
    }

    /// Sets the characters to be for word break.
    pub fn word_break_chars(mut self, characters: HashSet<char>) -> Self {
        self.readline.word_break_chars = characters;
        self
    }

    /// Sets the number of lines available for rendering the text editor.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.readline.lines = Some(lines);
        self
    }

    /// Sets the function to evaluate the input, allowing for custom evaluation logic.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
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
}
