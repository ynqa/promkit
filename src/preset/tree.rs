use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    render::{Renderable, State},
    style::Style,
    text,
    tree::{self, Node},
    Prompt,
};

pub struct Theme {
    /// Style for title (enabled if you set title).
    pub title_style: ContentStyle,

    /// Style for selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected line.
    pub inactive_item_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            active_item_style: Style::new().fgc(Color::DarkCyan).build(),
            inactive_item_style: Style::new().build(),
        }
    }
}

pub struct Tree {
    title: String,
    tree: tree::Tree,
    theme: Theme,
    folded_symbol: String,
    unfolded_symbol: String,
    window_size: Option<usize>,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            title: Default::default(),
            tree: tree::Tree::new(root),
            theme: Default::default(),
            folded_symbol: String::from("▶︎ "),
            unfolded_symbol: String::from("▼ "),
            window_size: Default::default(),
        }
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title = text.as_ref().to_string();
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn window_size(mut self, window_size: usize) -> Self {
        self.window_size = Some(window_size);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                State::<text::Renderer>::try_new(self.title, self.theme.title_style)?,
                State::<tree::Renderer>::try_new(
                    self.tree,
                    self.folded_symbol,
                    self.unfolded_symbol,
                    self.theme.active_item_style,
                    self.theme.inactive_item_style,
                    self.window_size,
                )?,
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<tree::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .tree
                    .get())
            },
        )
    }
}
