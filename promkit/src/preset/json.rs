//! Enables parsing and interaction with JSON data.

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
    widgets::{
        jsonstream::{self, format::RowFormatter, JsonStream},
        text::{self, Text},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the JSON preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Json = 1,
}

/// Represents a JSON preset for rendering JSON data and titles with customizable styles.
pub struct Json {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// State for the title text.
    pub title: text::State,
    /// State for the JSON data, including formatting and rendering options.
    pub json: jsonstream::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Json {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Json, self.json.create_pane(size.0, size.1)),
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
        self.render(size.0, size.1).await?;
        ret
    }

    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

impl Json {
    /// Creates a new JSON preset with the provided JSON stream.
    pub fn new(stream: JsonStream) -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            title: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            json: jsonstream::State {
                stream,
                formatter: RowFormatter {
                    curly_brackets_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    square_brackets_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    key_style: ContentStyle {
                        foreground_color: Some(Color::DarkBlue),
                        ..Default::default()
                    },
                    string_value_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    number_value_style: ContentStyle::default(),
                    boolean_value_style: ContentStyle::default(),
                    null_value_style: ContentStyle {
                        foreground_color: Some(Color::DarkGrey),
                        ..Default::default()
                    },
                    active_item_attribute: Attribute::Undercurled,
                    inactive_item_attribute: Attribute::Dim,
                    indent: 2,
                },
                lines: Default::default(),
            },
        }
    }

    /// Sets the title text for the JSON preset.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.style = style;
        self
    }

    /// Sets the number of lines to be used for rendering the JSON data.
    pub fn json_lines(mut self, lines: usize) -> Self {
        self.json.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the JSON data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.json.formatter.indent = indent;
        self
    }

    /// Sets the attribute for active (currently selected) items.
    pub fn active_item_attribute(mut self, attr: Attribute) -> Self {
        self.json.formatter.active_item_attribute = attr;
        self
    }

    /// Sets the attribute for inactive (not currently selected) items.
    pub fn inactive_item_attribute(mut self, attr: Attribute) -> Self {
        self.json.formatter.inactive_item_attribute = attr;
        self
    }

    /// Sets the evaluator function for handling events in the JSON preset.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }

    /// Render the prompt with the specified width and height.
    async fn render(&mut self, width: u16, height: u16) -> anyhow::Result<()> {
        match self.renderer.as_ref() {
            Some(renderer) => {
                renderer
                    .update([
                        (Index::Title, self.title.create_pane(width, height)),
                        (Index::Json, self.json.create_pane(width, height)),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}
