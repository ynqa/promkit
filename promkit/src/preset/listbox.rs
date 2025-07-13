//! Implements a list box for single or multiple selections from a list.

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
    widgets::{
        listbox,
        text::{self, Text},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the listbox preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Listbox = 1,
}

/// Type alias for the evaluator function used in the listbox preset.
pub type Evaluator = fn(event: &Event, ctx: &mut Listbox) -> anyhow::Result<Signal>;

/// A component for creating and managing a selectable list of options.
pub struct Listbox {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator_fn: Evaluator,
    /// State for the title displayed above the selectable list.
    pub title: text::State,
    /// State for the selectable list itself.
    pub listbox: listbox::State,
}

#[async_trait::async_trait]
impl crate::Prompt for Listbox {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Listbox, self.listbox.create_pane(size.0, size.1)),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = (self.evaluator_fn)(event, self);
        let size = crossterm::terminal::size()?;
        self.renderer
            .as_ref()
            .unwrap()
            .update([
                (Index::Title, self.title.create_pane(size.0, size.1)),
                (Index::Listbox, self.listbox.create_pane(size.0, size.1)),
            ])
            .render()
            .await?;
        ret
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.listbox.listbox.get().to_string())
    }
}

impl Listbox {
    /// Constructs a new `Listbox` instance
    /// with a list of items to be displayed as selectable options.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items
    ///   that implement the `Display` trait, to be used as options.
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            renderer: None,
            evaluator_fn: evaluate::default,
            title: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            listbox: listbox::State {
                listbox: listbox::Listbox::from_displayable(items),
                cursor: String::from("❯ "),
                active_item_style: Some(ContentStyle {
                    foreground_color: Some(Color::DarkCyan),
                    ..Default::default()
                }),
                inactive_item_style: Some(ContentStyle::default()),
                lines: Default::default(),
            },
        }
    }

    /// Sets the title text displayed above the selectable list.
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
        self.listbox.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox.active_item_style = Some(style);
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox.inactive_item_style = Some(style);
        self
    }

    /// Sets the number of lines to be used for displaying the selectable list.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox.lines = Some(lines);
        self
    }

    /// Sets the evaluator function for handling input events.
    pub fn evaluator(mut self, evaluator: Evaluator) -> Self {
        self.evaluator_fn = evaluator;
        self
    }
}
