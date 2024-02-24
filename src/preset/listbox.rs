use std::fmt::Display;

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    listbox,
    render::{Renderable, State},
    style::Style,
    text, Prompt,
};

/// A component for creating and managing a selectable list of options.
pub struct Listbox {
    /// Renderer for the title displayed above the selectable list.
    title_renderer: text::Renderer,
    /// Renderer for the selectable list itself.
    listbox_renderer: listbox::Renderer,
}

impl Listbox {
    /// Constructs a new `Listbox` instance
    /// with a list of items to be displayed as selectable options.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items
    /// that implement the `Display` trait, to be used as options.
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: listbox::Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                lines: Default::default(),
            },
        }
    }

    /// Sets the title text displayed above the selectable list.
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
        self.listbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the selectable list.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_renderer.lines = Some(lines);
        self
    }

    /// Displays the select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<listbox::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .listbox
                    .get())
            },
        )
    }
}
