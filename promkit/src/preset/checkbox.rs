//! Provides a checkbox interface for multiple options selection.

use std::fmt::Display;

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
        checkbox,
        text::{self, Text},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the checkbox preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Checkbox = 1,
}

/// Represents a checkbox component for creating
/// and managing a list of selectable options.
pub struct Checkbox {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// State for the title displayed above the checkbox list.
    pub title: text::State,
    /// State for the checkbox list itself.
    pub checkbox: checkbox::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Checkbox {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Checkbox, self.checkbox.create_pane(size.0, size.1)),
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

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .checkbox
            .checkbox
            .get()
            .iter()
            .map(|e| e.to_string())
            .collect())
    }
}

impl Checkbox {
    fn new_with_checkbox(checkbox: checkbox::Checkbox) -> Self {
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
            checkbox: checkbox::State {
                checkbox,
                cursor: String::from("❯ "),
                active_mark: '☒',
                inactive_mark: '☐',
                active_item_style: ContentStyle {
                    foreground_color: Some(Color::DarkCyan),
                    ..Default::default()
                },
                inactive_item_style: ContentStyle::default(),
                lines: Default::default(),
            },
        }
    }

    /// Creates a new `Checkbox` instance with the provided items.
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self::new_with_checkbox(checkbox::Checkbox::from_displayable(items))
    }

    /// Creates a new `Checkbox` instance with the provided items and their checked states.
    pub fn new_with_checked<T: Display, I: IntoIterator<Item = (T, bool)>>(items: I) -> Self {
        Self::new_with_checkbox(checkbox::Checkbox::new_with_checked(items))
    }

    /// Sets the title text displayed above the checkbox list.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.style = style;
        self
    }

    /// Sets the cursor symbol used to indicate the current selection.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.checkbox.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the mark symbol used to indicate selected items.
    pub fn active_mark(mut self, mark: char) -> Self {
        self.checkbox.active_mark = mark;
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the checkbox list.
    pub fn checkbox_lines(mut self, lines: usize) -> Self {
        self.checkbox.lines = Some(lines);
        self
    }

    /// Sets the evaluator function for handling input events.
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
                        (Index::Checkbox, self.checkbox.create_pane(width, height)),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}
