use std::{cell::RefCell, fmt::Display};

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    listbox,
    snapshot::Snapshot,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text, Prompt,
};

pub mod keymap;
pub mod render;

/// A component for creating and managing a selectable list of options.
pub struct Listbox {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// State for the title displayed above the selectable list.
    title_state: text::State,
    /// State for the selectable list itself.
    listbox_state: listbox::State,
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
            title_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            listbox_state: listbox::State {
                listbox: listbox::Listbox::from_displayable(items),
                cursor: String::from("❯ "),
                active_item_style: Some(StyleBuilder::new().fgc(Color::DarkCyan).build()),
                inactive_item_style: Some(StyleBuilder::new().build()),
                lines: Default::default(),
            },
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
        }
    }

    /// Sets the title text displayed above the selectable list.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_state.style = style;
        self
    }

    /// Sets the cursor symbol used to indicate the current selection.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.listbox_state.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_state.active_item_style = Some(style);
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_state.inactive_item_style = Some(style);
        self
    }

    /// Sets the number of lines to be used for displaying the selectable list.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_state.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(mut self, key: K, handler: keymap::Keymap) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_snapshot: Snapshot::<text::State>::new(self.title_state),
                listbox_snapshot: Snapshot::<listbox::State>::new(self.listbox_state),
            },
        })
    }
}
