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
    folded_cursor: String,
    unfolded_cursor: String,
    cursor_style: ContentStyle,
    lines: Option<usize>,
}

impl Builder {
    pub fn new(root: Node) -> Self {
        Self {
            tree: Tree::new(root),
            style: Default::default(),
            folded_cursor: Default::default(),
            unfolded_cursor: Default::default(),
            cursor_style: Default::default(),
            lines: Default::default(),
        }
    }

    pub fn folded_cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.folded_cursor = cursor.as_ref().to_string();
        self
    }

    pub fn unfolded_cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.unfolded_cursor = cursor.as_ref().to_string();
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
            folded_cursor: self.folded_cursor,
            unfolded_cursor: self.unfolded_cursor,
            style: self.style,
            cursor_style: self.cursor_style,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(self.build()?)))
    }
}
