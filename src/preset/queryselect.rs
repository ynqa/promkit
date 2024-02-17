use std::{fmt::Display, iter::FromIterator};

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    listbox::{self, Listbox},
    render::{Renderable, State},
    style::Style,
    text,
    text_editor::{self, Mode, Suggest},
    Prompt,
};

/// Used to process and filter a list of options
/// based on the input text in the `QuerySelect` component.
type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

/// Represents a query selection component that combines a text editor
/// for input and a list box
/// for displaying filtered options based on the input.
pub struct QuerySelect {
    /// Renderer for the title displayed above the query selection.
    title_renderer: text::Renderer,
    /// Renderer for the text editor component.
    text_editor_renderer: text_editor::Renderer,
    /// Renderer for the list box component.
    listbox_renderer: listbox::Renderer,
    /// A filter function to apply to the list box items
    /// based on the text editor input.
    filter: Box<Filter>,
}

impl QuerySelect {
    /// Constructs a new `QuerySelect` instance
    /// with a list of items and a filter function.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items that implement the `Display` trait,
    /// to be used as options in the list box.
    /// * `filter` - A function that takes the current input
    /// from the text editor and the list of items,
    /// returning a filtered list of items to display.
    pub fn new<T, I, F>(items: I, filter: F) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
        F: Fn(&str, &Vec<String>) -> Vec<String> + 'static,
    {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: None,
                suggest: Default::default(),
                ps: String::from("❯❯ "),
                mask: None,
                ps_style: Style::new().fgc(Color::DarkGreen).build(),
                active_char_style: Style::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: Style::new().build(),
                edit_mode: Default::default(),
                lines: Default::default(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                lines: Default::default(),
            },
            filter: Box::new(filter),
        }
    }

    /// Sets the title text displayed above the query selection.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Enables suggestion functionality in the text editor component with the specified suggestion configuration.
    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_renderer.suggest = suggest;
        self
    }

    /// Sets the prefix string displayed before the input text in the text editor component.
    pub fn prefix_string<T: AsRef<str>>(mut self, ps: T) -> Self {
        self.text_editor_renderer.ps = ps.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string in the text editor component.
    pub fn prefix_string_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.ps_style = style;
        self
    }

    /// Sets the style for the active character (the character at the cursor position) in the text editor component.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.active_char_style = style;
        self
    }

    /// Sets the style for inactive characters (characters not at the cursor position) in the text editor component.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.inactive_char_style = style;
        self
    }

    /// Sets the editing mode for the text editor component.
    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_renderer.edit_mode = mode;
        self
    }

    /// Sets the number of lines available for the text editor component.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_renderer.lines = Some(lines);
        self
    }

    /// Sets the cursor symbol used in the list box component.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.listbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items in the list box component.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items in the list box component.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines available for the list box component.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_renderer.lines = Some(lines);
        self
    }

    /// Displays the query select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(State::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            move |_: &Event, renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<bool> {
                let text_editor_state = renderables[1]
                    .as_any()
                    .downcast_ref::<State<text_editor::Renderer>>()
                    .unwrap();
                let select_state = renderables[2]
                    .as_any()
                    .downcast_ref::<State<listbox::Renderer>>()
                    .unwrap();

                if text_editor_state.text_changed() {
                    let query = text_editor_state
                        .after
                        .borrow()
                        .texteditor
                        .text_without_cursor();

                    let list = filter(&query, select_state.init.listbox.items());
                    select_state.after.borrow_mut().listbox = Listbox::from_iter(list);
                }
                Ok(true)
            },
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[2]
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
