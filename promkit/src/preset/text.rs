//! Provides a static text display.

use crate::{
    core::{
        crossterm::{self, event::Event, style::ContentStyle},
        render::{Renderer, SharedRenderer},
        PaneFactory,
    },
    widgets::text,
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the text preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Text = 0,
}

/// Type alias for the evaluator function used in the `Text` prompt.
pub type Evaluator = fn(event: &Event, ctx: &mut Text) -> anyhow::Result<Signal>;

/// Represents a text component for displaying static text in a prompt.
pub struct Text {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: SharedRenderer<Index>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator_fn: Evaluator,
    /// Text state containing the text to be displayed.
    pub text: text::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Text {
    type Index = Index;

    fn renderer(&self) -> SharedRenderer<Self::Index> {
        self.renderer.clone()
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator_fn)(event, self);
        let size = crossterm::terminal::size()?;
        self.renderer
            .as_ref()
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
    /// Creates a new `Text` prompt with the given text and evaluator function.
    pub async fn try_default<T: AsRef<str>>(text: T) -> anyhow::Result<Self> {
        let size = crossterm::terminal::size()?;

        let text = text::State {
            text: text::Text::from(text),
            style: Default::default(),
            lines: None,
        };

        Ok(Self {
            renderer: SharedRenderer::new(
                Renderer::try_new_with_panes(
                    [(Index::Text, text.create_pane(size.0, size.1))],
                    true,
                )
                .await?,
            ),
            evaluator_fn: evaluate::default,
            text,
        })
    }

    /// Sets the style for the text component.
    pub fn style(mut self, style: ContentStyle) -> Self {
        self.text.style = style;
        self
    }

    /// Sets the evaluator function for the text prompt.
    pub fn evaluator<K: AsRef<str>>(mut self, evaluator: Evaluator) -> Self {
        self.evaluator_fn = evaluator;
        self
    }
}
