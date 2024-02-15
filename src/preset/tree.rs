use crate::{
    error::Result,
    preset::theme::tree::Theme,
    render::{Renderable, State},
    text,
    tree::{self, Node},
    Prompt,
};

pub struct Tree {
    title_builder: text::Builder,
    tree_builder: tree::Builder,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            title_builder: Default::default(),
            tree_builder: tree::Builder::new(root),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.tree_builder = self
            .tree_builder
            .folded_symbol(theme.folded_symbol)
            .unfolded_symbol(theme.unfolded_symbol)
            .style(theme.item_style)
            .cursor_style(theme.cursor_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_builder = self.title_builder.text(text);
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.tree_builder = self.tree_builder.lines(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                self.title_builder.build_state()?,
                self.tree_builder.build_state()?,
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
