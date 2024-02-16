use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    render::{Renderable, State},
    style::Style,
    text,
    tree::{self, Node},
    Prompt,
};

pub struct Tree {
    title_renderer: text::Renderer,
    tree_renderer: tree::Renderer,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            tree_renderer: tree::Renderer {
                tree: tree::Tree::new(root),
                folded_symbol: String::from("▶︎ "),
                unfolded_symbol: String::from("▼ "),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                screen_lines: Default::default(),
            },
        }
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    pub fn folded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree_renderer.folded_symbol = symbol.as_ref().to_string();
        self
    }

    pub fn unfolded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.tree_renderer.unfolded_symbol = symbol.as_ref().to_string();
        self
    }

    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.tree_renderer.active_item_style = style;
        self
    }

    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.tree_renderer.inactive_item_style = style;
        self
    }

    pub fn screen_lines(mut self, screen_lines: usize) -> Self {
        self.tree_renderer.screen_lines = Some(screen_lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<tree::Renderer>::new(self.tree_renderer)),
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<Vec<String>> {
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
