use crate::{
    error::Result,
    preset::theme::tree::Theme,
    tree::Node,
    view::{State, TextViewerBuilder, TreeViewer, TreeViewerBuilder, Viewable},
    Prompt,
};

pub struct Tree {
    title_builder: TextViewerBuilder,
    tree_builder: TreeViewerBuilder,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            title_builder: Default::default(),
            tree_builder: TreeViewerBuilder::new(root),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.tree_builder = self
            .tree_builder
            .cursor(theme.cursor)
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
            |viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<String> {
                Ok(viewables[1]
                    .as_any()
                    .downcast_ref::<State<TreeViewer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .tree
                    .get())
            },
        )
    }
}
