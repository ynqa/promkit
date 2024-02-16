mod node;

use crate::core::cursor::Cursor;

pub use node::{Node, NodeWithDepth};
mod render;
pub use render::Renderer;

#[derive(Clone)]
pub struct Tree {
    root: Node,
    cursor: Cursor<Vec<NodeWithDepth>>,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root: root.clone(),
            cursor: Cursor::new(root.flatten()),
        }
    }

    pub fn nodes(&self) -> Vec<NodeWithDepth> {
        self.cursor.contents().clone()
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn get(&self) -> String {
        self.cursor
            .contents()
            .get(self.position())
            .unwrap()
            .data
            .clone()
    }

    pub fn toggle(&mut self) {
        self.root.toggle(self.cursor.position());
        self.cursor = Cursor::new_with_position(self.root.flatten(), self.position());
    }

    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }
}
