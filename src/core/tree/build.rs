use crate::{
    crossterm::style::ContentStyle,
    error::Result,
    render::State,
    tree::{Node, Renderer, Tree},
};

#[derive(Clone)]
pub struct Builder {
    tree: Tree,
    style: ContentStyle,
    folded_symbol: String,
    unfolded_symbol: String,
    cursor_style: ContentStyle,
    lines: Option<usize>,
}

impl Builder {
    pub fn new(root: Node) -> Self {
        Self {
            tree: Tree::new(root),
            style: Default::default(),
            folded_symbol: Default::default(),
            unfolded_symbol: Default::default(),
            cursor_style: Default::default(),
            lines: Default::default(),
        }
    }

    pub fn folded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.folded_symbol = symbol.as_ref().to_string();
        self
    }

    pub fn unfolded_symbol<T: AsRef<str>>(mut self, symbol: T) -> Self {
        self.unfolded_symbol = symbol.as_ref().to_string();
        self
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: ContentStyle) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = Some(lines);
        self
    }

    pub fn build(self) -> Result<Renderer> {
        Ok(Renderer {
            tree: self.tree,
            folded_symbol: self.folded_symbol,
            unfolded_symbol: self.unfolded_symbol,
            style: self.style,
            cursor_style: self.cursor_style,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(self.build()?)))
    }
}
