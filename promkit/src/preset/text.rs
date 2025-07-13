//! Provides a static text display.

use crate::{
    core::{
        crossterm::{self, event::Event, style::ContentStyle},
        render::{Renderer, SharedRenderer},
        PaneFactory,
    },
    preset::Evaluator,
    widgets::text,
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the text preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Text = 0,
}

/// Represents a text component for displaying static text in a prompt.
pub struct Text {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// Text state containing the text to be displayed.
    pub text: text::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Text {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [(Index::Text, self.text.create_pane(size.0, size.1))],
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
            .update([(Index::Text, self.text.create_pane(size.0, size.1))])
            .render()
            .await?;
        ret
    }

    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

impl Text {
    /// Creates a new `Text` instance with the provided text.
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self {
            renderer: None,
            evaluator: |event, ctx| Box::pin(evaluate::default(event, ctx)),
            text: text::State {
                text: text::Text::from(text),
                style: Default::default(),
                lines: None,
            },
        }
    }

    /// Sets the style for the text component.
    pub fn style(mut self, style: ContentStyle) -> Self {
        self.text.style = style;
        self
    }

    /// Sets the evaluator function for the text prompt.
    pub fn evaluator(mut self, evaluator: Evaluator<Self>) -> Self {
        self.evaluator = evaluator;
        self
    }
}
