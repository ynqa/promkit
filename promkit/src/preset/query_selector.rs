//! Facilitates querying and selecting from a set of options in a structured format.

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
        listbox::{self, Listbox},
        text::{self, Text},
        text_editor::{self, Mode},
    },
    Signal,
};

pub mod evaluate;

/// Represents the indices of various components in the query selector preset.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Index {
    Title = 0,
    Readline = 1,
    List = 2,
}

/// Used to process and filter a list of options
/// based on the input text in the `QuerySelector` component.
pub type Filter = fn(&str, &Vec<String>) -> Vec<String>;

/// Represents a query selection component that combines a text editor
/// for input and a list box
/// for displaying filtered options based on the input.
pub struct QuerySelector {
    /// Shared renderer for the prompt, allowing for rendering of UI components.
    pub renderer: Option<SharedRenderer<Index>>,
    /// Function to evaluate the input events and update the state of the prompt.
    pub evaluator: Evaluator<Self>,
    /// State for the title displayed above the query selection.
    pub title: text::State,
    /// State for the text editor component.
    pub readline: text_editor::State,
    /// Initial state for the list box component.
    pub init_list: Listbox,
    /// State for the list box component.
    pub list: listbox::State,
    /// A filter function to apply to the list box items
    /// based on the text editor input.
    pub filter: Filter,
}

#[async_trait::async_trait]
impl crate::Prompt for QuerySelector {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let size = crossterm::terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_panes(
                [
                    (Index::Title, self.title.create_pane(size.0, size.1)),
                    (Index::Readline, self.readline.create_pane(size.0, size.1)),
                    (Index::List, self.list.create_pane(size.0, size.1)),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        // Store the previous text in the readline before evaluating the event.
        let prev = self.readline.texteditor.text_without_cursor().to_string();

        // Evaluate the event using the provided evaluator function.
        let ret = (self.evaluator)(event, self).await;

        // If the text in the readline has changed, we need to filter the list.
        if prev != self.readline.texteditor.text_without_cursor().to_string() {
            let query = self.readline.texteditor.text_without_cursor().to_string();
            let list = (self.filter)(
                &query,
                &self
                    .init_list
                    .items()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            );
            self.list.listbox = Listbox::from_displayable(list);
        }

        // Update the renderer with the new state of the components.
        let size = crossterm::terminal::size()?;
        self.render(size.0, size.1).await?;

        ret
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.list.listbox.get().to_string())
    }
}

impl QuerySelector {
    /// Constructs a new `QuerySelector` instance
    /// with a list of items and a filter function.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items that implement the `Display` trait,
    ///   to be used as options in the list box.
    /// * `filter` - A function that takes the current input
    ///   from the text editor and the list of items,
    ///   returning a filtered list of items to display.
    pub fn new<T, I>(items: I, filter: Filter) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
    {
        let listbox = Listbox::from_displayable(items);
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
            readline: text_editor::State {
                texteditor: Default::default(),
                history: None,
                prefix: String::from("❯❯ "),
                mask: None,
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
                word_break_chars: Default::default(),
                lines: Default::default(),
            },
            init_list: listbox.clone(),
            list: listbox::State {
                listbox,
                cursor: String::from("❯ "),
                active_item_style: Some(ContentStyle {
                    foreground_color: Some(Color::DarkCyan),
                    ..Default::default()
                }),
                inactive_item_style: Some(ContentStyle::default()),
                lines: Default::default(),
            },
            filter,
        }
    }

    /// Sets the title text displayed above the query selection.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title.style = style;
        self
    }

    /// Sets the prefix string displayed before the input text in the text editor component.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.readline.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string in the text editor component.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.readline.prefix_style = style;
        self
    }

    /// Sets the style for the active character (the character at the cursor position) in the text editor component.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.active_char_style = style;
        self
    }

    /// Sets the style for inactive characters (characters not at the cursor position) in the text editor component.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.readline.inactive_char_style = style;
        self
    }

    /// Sets the editing mode for the text editor component.
    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.readline.edit_mode = mode;
        self
    }

    /// Sets the number of lines available for the text editor component.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.readline.lines = Some(lines);
        self
    }

    /// Sets the cursor symbol used in the list box component.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.list.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items in the list box component.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.list.active_item_style = Some(style);
        self
    }

    /// Sets the style for inactive (not currently selected) items in the list box component.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.list.inactive_item_style = Some(style);
        self
    }

    /// Sets the number of lines available for the list box component.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.list.lines = Some(lines);
        self
    }

    /// Sets the evaluator function for the text prompt.
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
                        (Index::Readline, self.readline.create_pane(width, height)),
                        (Index::List, self.list.create_pane(width, height)),
                    ])
                    .render()
                    .await
            }
            None => Err(anyhow::anyhow!("Renderer not initialized")),
        }
    }
}
