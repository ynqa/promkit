use std::{cell::RefCell, fmt::Display};

use crate::{
    checkbox,
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    snapshot::Snapshot,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text, Prompt,
};

pub mod keymap;
pub mod render;

/// Represents a checkbox component for creating
/// and managing a list of selectable options.
pub struct Checkbox {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// State for the title displayed above the checkbox list.
    title_state: text::State,
    /// State for the checkbox list itself.
    checkbox_state: checkbox::State,
}

impl Checkbox {
    /// Constructs a new `Checkbox` instance with a list of items
    /// to be displayed as selectable options.
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
            checkbox_state: checkbox::State {
                checkbox: checkbox::Checkbox::from_displayable(items),
                cursor: String::from("❯ "),
                active_mark: '☒',
                inactive_mark: '☐',
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
        }
    }

    pub fn new_with_checked<T: Display, I: IntoIterator<Item = (T, bool)>>(items: I) -> Self {
        Self {
            title_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            checkbox_state: checkbox::State {
                checkbox: checkbox::Checkbox::new_with_checked(items),
                cursor: String::from("❯ "),
                active_mark: '☒',
                inactive_mark: '☐',
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
        }
    }

    /// Sets the title text displayed above the checkbox list.
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
        self.checkbox_state.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the mark symbol used to indicate selected items.
    pub fn active_mark(mut self, mark: char) -> Self {
        self.checkbox_state.active_mark = mark;
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_state.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_state.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the checkbox list.
    pub fn checkbox_lines(mut self, lines: usize) -> Self {
        self.checkbox_state.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(mut self, key: K, handler: keymap::Keymap) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the checkbox prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is a list of selected options.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_snapshot: Snapshot::<text::State>::new(self.title_state),
                checkbox_snapshot: Snapshot::<checkbox::State>::new(self.checkbox_state),
            },
        })
    }
}
