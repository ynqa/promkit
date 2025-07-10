//! Supports creating and interacting with a tree structure for hierarchical data.

use std::cell::RefCell;

use promkit_widgets::{
    text::{self, Text},
    tree::{self, node::Node},
};

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    switch::ActiveKeySwitcher,
    Prompt,
};

pub mod keymap;
pub mod render;

/// Represents a tree component for creating
/// and managing a hierarchical list of options.
pub struct Tree {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    /// State for the title displayed above the tree.
    title_state: text::State,
    /// State for the tree itself.
    tree_state: tree::State,
}

impl Tree {
    /// Constructs a new `Tree` instance with a specified root node.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node of the tree.
    pub fn new(root: Node) -> Self {
        Self {
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
            title_state: text::State {
                style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                ..Default::default()
            },
            tree_state: tree::State {
                tree: tree::Tree::new(root),
                folded_symbol: String::from("▶︎ "),
                unfolded_symbol: String::from("▼ "),
                active_item_style: ContentStyle {
                    foreground_color: Some(Color::DarkCyan),
                    ..Default::default()
                },
                inactive_item_style: ContentStyle::default(),
                lines: Default::default(),
                indent: 2,
            },
        }
    }

    /// Sets the title text displayed above the tree.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = Text::from(text);
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_state.style = style;
        self
    }

    /// Sets the symbol used to indicate a folded (collapsed) node.
    pub fn folded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree_state.folded_symbol = symbol.as_ref().to_string();
        self
    }

    /// Sets the symbol used to indicate an unfolded (expanded) node.
    pub fn unfolded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree_state.unfolded_symbol = symbol.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.tree_state.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.tree_state.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the tree.
    pub fn tree_lines(mut self, lines: usize) -> Self {
        self.tree_state.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the tree data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.tree_state.indent = indent;
        self
    }

    pub fn register_keymap<K: AsRef<str>>(mut self, key: K, handler: keymap::Keymap) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the tree prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is a list of selected options.
    pub fn prompt(self) -> anyhow::Result<Prompt<render::Renderer>> {
        Ok(Prompt {
            renderer: render::Renderer {
                keymap: RefCell::new(self.keymap),
                title_state: self.title_state,
                tree_state: self.tree_state,
            },
        })
    }
}
