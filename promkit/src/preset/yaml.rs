//! Enables parsing and interaction with YAML data.

use crate::{
    core::{
        crossterm::{
            event::Event,
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    preset::Evaluator,
    widgets::{
        text::{self, Text},
        yaml::{
            self,
            config::{Config, OverflowMode},
            Document,
        },
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the YAML preset.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Yaml = 1,
}

/// Represents a YAML preset for rendering YAML data and titles with customizable styles.
pub struct Yaml {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// State for the title text.
    pub title: text::State,
    /// State for the YAML data, including formatting and rendering options.
    pub yaml: yaml::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Yaml {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::Yaml, self.yaml.create_graphemes()),
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

    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

impl Yaml {
    /// Creates a new YAML preset with the provided YAML document.
    pub fn new(document: Document) -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            title: text::State {
                config: text::config::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            yaml: yaml::State {
                document,
                config: Config {
                    map_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    sequence_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    key_style: ContentStyle {
                        foreground_color: Some(Color::DarkBlue),
                        ..Default::default()
                    },
                    tag_style: ContentStyle {
                        foreground_color: Some(Color::DarkYellow),
                        ..Default::default()
                    },
                    string_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    number_style: ContentStyle::default(),
                    boolean_style: ContentStyle::default(),
                    null_style: ContentStyle {
                        foreground_color: Some(Color::DarkGrey),
                        ..Default::default()
                    },
                    active_item_attribute: Attribute::Undercurled,
                    inactive_item_attribute: Attribute::Dim,
                    indent: 2,
                    overflow_mode: OverflowMode::Truncate,
                    lines: Default::default(),
                },
            },
        }
    }

    /// Sets the title text for the YAML preset.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.config.style = Some(style);
        self
    }

    /// Sets the number of lines to be used for rendering the YAML data.
    pub fn yaml_lines(mut self, lines: usize) -> Self {
        self.yaml.config.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the YAML data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.yaml.config.indent = indent;
        self
    }

    /// Sets the overflow mode for rendering YAML values that exceed the available width.
    pub fn overflow_mode(mut self, mode: OverflowMode) -> Self {
        self.yaml.config.overflow_mode = mode;
        self
    }

    /// Sets the attribute for active (currently selected) items.
    pub fn active_item_attribute(mut self, attr: Attribute) -> Self {
        self.yaml.config.active_item_attribute = attr;
        self
    }

    /// Sets the attribute for inactive (not currently selected) items.
    pub fn inactive_item_attribute(mut self, attr: Attribute) -> Self {
        self.yaml.config.inactive_item_attribute = attr;
        self
    }

    /// Sets the evaluator function for handling events in the YAML preset.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }

    /// Render the prompt with the specified width and height.
    async fn render(&mut self) -> anyhow::Result<()> {
        match self.renderer.as_ref() {
            Some(renderer) => {
                renderer
                    .update([
                        (Index::Title, self.title.create_graphemes()),
                        (Index::Yaml, self.yaml.create_graphemes()),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}
