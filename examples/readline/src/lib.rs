//! A complete readline prompt composed from promkit widgets.

use std::{collections::HashSet, future::Future, pin::Pin};

use promkit::{
    core::{
        crossterm::{
            event::Event,
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    suggest::Suggest,
    validate::{ErrorMessageGenerator, Validator, ValidatorManager},
    widgets::{
        listbox::{self, Listbox},
        text::{self, Text},
        text_editor::{self, History},
    },
    Prompt, Signal,
};

pub mod evaluate;

pub type Evaluator<T> =
    for<'a> fn(
        event: &'a Event,
        ctx: &'a mut T,
    ) -> Pin<Box<dyn Future<Output = Result<Signal, anyhow::Error>> + Send + 'a>>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Readline = 1,
    Suggestion = 2,
    ErrorMessage = 3,
}

pub enum Focus {
    Readline,
    Suggestion,
}

pub struct Readline {
    pub renderer: Option<SharedRenderer<Index>>,
    pub evaluator: Evaluator<Self>,
    pub focus: Focus,
    pub title: text::State,
    pub readline: text_editor::State,
    pub suggest: Option<Suggest>,
    pub suggestions: listbox::State,
    pub validator: Option<ValidatorManager<str>>,
    pub error_message: text::State,
}

impl Default for Readline {
    fn default() -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            focus: Focus::Readline,
            title: text::State {
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            readline: text_editor::State {
                texteditor: Default::default(),
                history: Default::default(),
                config: text_editor::Config {
                    prefix: String::from("❯❯ "),
                    continuation_prefix: Default::default(),
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
            },
            suggest: Default::default(),
            suggestions: listbox::State {
                listbox: Listbox::from(Vec::<String>::new()),
                config: listbox::Config {
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
            },
            validator: Default::default(),
            error_message: text::State {
                text: Default::default(),
                config: text::Config {
                    style: Some(ContentStyle {
                        foreground_color: Some(Color::DarkRed),
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    lines: None,
                },
            },
        }
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for Readline {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::Readline, self.readline.create_graphemes()),
                    (Index::Suggestion, self.suggestions.create_graphemes()),
                    (Index::ErrorMessage, self.error_message.create_graphemes()),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator)(event, self).await;
        self.render().await?;
        ret
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        let ret = self.readline.texteditor.text_without_cursor().to_string();
        self.readline.texteditor.erase_all();
        Ok(ret)
    }
}

impl Readline {
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.config.style = Some(style);
        self
    }

    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = Some(suggest);
        self
    }

    pub fn enable_history(mut self) -> Self {
        self.readline.history = Some(History::default());
        self
    }

    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.readline.config.prefix = prefix.as_ref().to_string();
        self
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.readline.config.mask = Some(mask);
        self
    }

    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.readline.config.prefix_style = style;
        self
    }

    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.config.active_char_style = style;
        self
    }

    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.config.inactive_char_style = style;
        self
    }

    pub fn edit_mode(mut self, mode: text_editor::Mode) -> Self {
        self.readline.config.edit_mode = mode;
        self
    }

    pub fn word_break_chars(mut self, characters: HashSet<char>) -> Self {
        self.readline.config.word_break_chars = characters;
        self
    }

    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.readline.config.lines = Some(lines);
        self
    }

    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }

    pub fn validator(
        mut self,
        validator: Validator<str>,
        error_message_generator: ErrorMessageGenerator<str>,
    ) -> Self {
        self.validator = Some(ValidatorManager::new(validator, error_message_generator));
        self
    }

    async fn render(&mut self) -> anyhow::Result<()> {
        match self.renderer.as_ref() {
            Some(renderer) => {
                renderer
                    .update([
                        (Index::Title, self.title.create_graphemes()),
                        (Index::Readline, self.readline.create_graphemes()),
                        (Index::Suggestion, self.suggestions.create_graphemes()),
                        (Index::ErrorMessage, self.error_message.create_graphemes()),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}
