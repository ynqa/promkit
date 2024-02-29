use std::{fmt::Display, iter::FromIterator};

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    listbox::{self, Listbox},
    snapshot::Snapshot,
    style::StyleBuilder,
    text,
    text_editor::{self, Mode, Suggest},
    Prompt, Renderer,
};

/// Used to process and filter a list of options
/// based on the input text in the `QuerySelector` component.
type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

/// Represents a query selection component that combines a text editor
/// for input and a list box
/// for displaying filtered options based on the input.
pub struct QuerySelector {
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

impl QuerySelector {
    /// Constructs a new `QuerySelector` instance
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
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: None,
                suggest: Default::default(),
                keymap: KeymapManager::new(text_editor::keymap::default_keymap()),
                prefix: String::from("❯❯ "),
                mask: None,
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                lines: Default::default(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
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
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_renderer.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string in the text editor component.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.prefix_style = style;
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
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(Snapshot::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            move |_: &Event, renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<bool> {
                let text_editor_state = renderers[1]
                    .as_any()
                    .downcast_ref::<Snapshot<text_editor::Renderer>>()
                    .unwrap();
                let select_state = renderers[2]
                    .as_any()
                    .downcast_ref::<Snapshot<listbox::Renderer>>()
                    .unwrap();

                if text_editor_state.text_changed() {
                    let query = text_editor_state
                        .after
                        .borrow()
                        .texteditor
                        .text_without_cursor()
                        .to_string();

                    let list = filter(&query, select_state.init.listbox.items());
                    select_state.after.borrow_mut().listbox = Listbox::from_iter(list);
                }
                Ok(true)
            },
            |renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<String> {
                Ok(renderers[2]
                    .as_any()
                    .downcast_ref::<Snapshot<listbox::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .listbox
                    .get())
            },
        )
    }
}