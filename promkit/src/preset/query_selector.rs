use std::{cell::RefCell, fmt::Display};

use promkit_widgets::{
    listbox::{self, Listbox},
    text::{self, Text},
    text_editor::{self, Mode},
};

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    snapshot::Snapshot,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    Prompt,
};

pub mod keymap;
pub mod render;

/// Represents a query selection component that combines a text editor
/// for input and a list box
/// for displaying filtered options based on the input.
pub struct QuerySelector {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// State for the title displayed above the query selection.
    title_state: text::State,
    /// State for the text editor component.
    text_editor_state: text_editor::State,
    /// State for the list box component.
    listbox_state: listbox::State,
    /// A filter function to apply to the list box items
    /// based on the text editor input.
    filter: render::Filter,
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
    pub fn new<T, I>(items: I, filter: render::Filter) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
    {
        Self {
            title_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
                lines: None,
            },
            text_editor_state: text_editor::State {
                texteditor: Default::default(),
                history: None,
                prefix: String::from("❯❯ "),
                mask: None,
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                word_break_chars: Default::default(),
                lines: Default::default(),
            },
            listbox_state: listbox::State {
                listbox: Listbox::from_displayable(items),
                cursor: String::from("❯ "),
                active_item_style: Some(StyleBuilder::new().fgc(Color::DarkCyan).build()),
                inactive_item_style: Some(StyleBuilder::new().build()),
                lines: Default::default(),
            },
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
            filter,
        }
    }

    /// Sets the title text displayed above the query selection.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_state.style = style;
        self
    }

    /// Sets the prefix string displayed before the input text in the text editor component.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_state.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string in the text editor component.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.prefix_style = style;
        self
    }

    /// Sets the style for the active character (the character at the cursor position) in the text editor component.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.active_char_style = style;
        self
    }

    /// Sets the style for inactive characters (characters not at the cursor position) in the text editor component.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_state.inactive_char_style = style;
        self
    }

    /// Sets the editing mode for the text editor component.
    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_state.edit_mode = mode;
        self
    }

    /// Sets the number of lines available for the text editor component.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_state.lines = Some(lines);
        self
    }

    /// Sets the cursor symbol used in the list box component.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.listbox_state.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items in the list box component.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_state.active_item_style = Some(style);
        self
    }

    /// Sets the style for inactive (not currently selected) items in the list box component.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_state.inactive_item_style = Some(style);
        self
    }

    /// Sets the number of lines available for the list box component.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_state.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(mut self, key: K, handler: keymap::Keymap) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the query select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_state: self.title_state,
                text_editor_snapshot: Snapshot::<text_editor::State>::new(self.text_editor_state),
                listbox_snapshot: Snapshot::<listbox::State>::new(self.listbox_state),
                filter: self.filter,
            },
        })
    }
}
