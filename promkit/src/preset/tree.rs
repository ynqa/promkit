use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    snapshot::Snapshot,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text,
    tree::{self, Node},
    EventHandler, Prompt, Renderer,
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
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            tree_state: tree::State {
                tree: tree::Tree::new(root),
                folded_symbol: String::from("▶︎ "),
                unfolded_symbol: String::from("▼ "),
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
                indent: 2,
            },
        }
    }

    /// Sets the title text displayed above the tree.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = text.as_ref().to_string();
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

    pub fn register_keymap<K: AsRef<str>>(
        mut self,
        key: K,
        handler: EventHandler<self::render::Renderer>,
    ) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the tree prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is a list of selected options.
    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            Box::new(self::render::Renderer {
                keymap: self.keymap,
                title_snapshot: Snapshot::<text::State>::new(self.title_state),
                tree_snapshot: Snapshot::<tree::State>::new(self.tree_state),
            }),
            Box::new(
                |event: &Event, renderer: &mut Box<dyn Renderer + 'static>| {
                    let renderer = self::render::Renderer::cast_mut(renderer.as_mut())?;
                    renderer.keymap.get()(event, renderer)
                },
            ),
            |renderer: &(dyn Renderer + '_)| -> Result<Vec<String>> {
                Ok(self::render::Renderer::cast(renderer)?
                    .tree_snapshot
                    .after()
                    .tree
                    .get())
            },
        )
    }
}
