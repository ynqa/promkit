mod node;
pub use node::{Node, NodeWithDepth};
mod render;
pub use render::Renderer;
mod build;
pub use build::Builder;

#[derive(Clone, Default)]
pub struct Tree {
    pub root: Node,
    pub cache: Vec<NodeWithDepth>,
    pub position: usize,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root: root.clone(),
            cache: root.flatten(),
            position: 0,
        }
    }

    pub fn content(&self) -> Vec<NodeWithDepth> {
        self.cache.clone()
    }

    pub fn get(&self) -> String {
        self.cache.get(self.position).unwrap().data.clone()
    }

    pub fn backward(&mut self) -> bool {
        if 0 < self.position {
            self.position -= 1;
            return true;
        }
        false
    }

    pub fn forward(&mut self) -> bool {
        if !self.cache.is_empty() && self.position < self.cache.len() - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    pub fn toggle(&mut self) {
        self.root.toggle(self.position);
        self.cache = self.root.flatten();
    }
}
