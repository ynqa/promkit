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
    validate::ValidatorManager,
    widgets::{
        listbox::{self, Listbox},
        text, text_editor,
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
