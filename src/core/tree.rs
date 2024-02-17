mod node;

use crate::core::cursor::Cursor;

pub use node::{Node, NodeWithDepth};
mod render;
pub use render::Renderer;

/// A `Tree` structure that manages a collection of nodes in a hierarchical manner.
/// It utilizes a cursor to navigate and manipulate the nodes within the tree.
#[derive(Clone)]
pub struct Tree {
    root: Node,
    cursor: Cursor<Vec<NodeWithDepth>>,
}

impl Tree {
    /// Creates a new `Tree` with a given root node.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node of the tree.
    pub fn new(root: Node) -> Self {
        Self {
            root: root.clone(),
            cursor: Cursor::new(root.flatten()),
        }
    }

    /// Returns a vector of all nodes in the tree, represented with their depth information.
    pub fn nodes(&self) -> Vec<NodeWithDepth> {
        self.cursor.contents().clone()
    }

    /// Returns the current position of the cursor within the tree.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    /// Retrieves the data of the current node pointed by the cursor, along with its path from the root.
    pub fn get(&self) -> Vec<String> {
        let node = self.cursor.contents().get(self.position()).unwrap();

        let mut ret = node.data_from_root.clone();
        ret.push(node.data.clone());
        ret
    }

    /// Toggles the state of the current node and updates the cursor position accordingly.
    pub fn toggle(&mut self) {
        self.root.toggle(self.cursor.position());
        self.cursor = Cursor::new_with_position(self.root.flatten(), self.position());
    }

    /// Moves the cursor backward in the tree, if possible.
    ///
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    /// Moves the cursor forward in the tree, if possible.
    ///
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the cursor to the head of the tree.
    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }
}
