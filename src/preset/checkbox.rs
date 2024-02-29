use std::fmt::Display;

use crate::{
    checkbox,
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    snapshot::Snapshot,
    style::StyleBuilder,
    text, Prompt, Renderer,
};

/// Represents a checkbox component for creating
/// and managing a list of selectable options.
pub struct Checkbox {
    /// Renderer for the title displayed above the checkbox list.
    title_renderer: text::Renderer,
    /// Renderer for the checkbox list itself.
    checkbox_renderer: checkbox::Renderer,
}

impl Checkbox {
    /// Constructs a new `Checkbox` instance with a list of items
    /// to be displayed as selectable options.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items
    /// that implement the `Display` trait, to be used as options.
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            checkbox_renderer: checkbox::Renderer {
                checkbox: checkbox::Checkbox::from_iter(items),
                cursor: String::from("❯ "),
                mark: '■',
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
        }
    }

    /// Sets the title text displayed above the checkbox list.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Sets the cursor symbol used to indicate the current selection.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.checkbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the mark symbol used to indicate selected items.
    pub fn mark(mut self, mark: char) -> Self {
        self.checkbox_renderer.mark = mark;
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the checkbox list.
    pub fn checkbox_lines(mut self, lines: usize) -> Self {
        self.checkbox_renderer.lines = Some(lines);
        self
    }

    /// Displays the checkbox prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is a list of selected options.
    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            vec![
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<checkbox::Renderer>::new(self.checkbox_renderer)),
            ],
            |_, _| Ok(true),
            |renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<Vec<String>> {
                Ok(renderers[1]
                    .as_any()
                    .downcast_ref::<Snapshot<checkbox::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .checkbox
                    .get())
            },
        )
    }
}
