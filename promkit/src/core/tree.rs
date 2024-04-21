mod node;

use crate::core::cursor::Cursor;

pub use node::{Kind, Node, Path};
mod state;
pub use state::State;

/// A `Tree` structure that manages a collection of nodes in a hierarchical manner.
/// It utilizes a cursor to navigate and manipulate the nodes within the tree.
#[derive(Clone)]
pub struct Tree {
    root: Node,
    cursor: Cursor<Vec<Kind>>,
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
            cursor: Cursor::new(root.flatten_visibles(), 0, false),
        }
    }

    /// Returns a vector of all nodes in the tree, represented with their depth information.
    pub fn kinds(&self) -> Vec<Kind> {
        self.cursor.contents().clone()
    }

    /// Returns the current position of the cursor within the tree.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    /// Retrieves the data of the current node pointed by the cursor, along with its path from the root.
    pub fn get(&self) -> Vec<String> {
        let kind = self.cursor.contents()[self.position()].clone();
        match kind {
            Kind::Folded { id, path } | Kind::Unfolded { id, path } => {
                let mut ret = self.root.get_waypoints(&path);
                ret.push(id.to_string());
                ret
            }
        }
    }

    /// Toggles the state of the current node and updates the cursor position accordingly.
    pub fn toggle(&mut self) {
        let path = match self.cursor.contents()[self.position()].clone() {
            Kind::Folded { path, .. } => path,
            Kind::Unfolded { path, .. } => path,
        };
        self.root.toggle(&path);
        self.cursor = Cursor::new(self.root.flatten_visibles(), self.position(), false);
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

    /// Moves the cursor to the tail of the tree.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
    }

    pub fn viewport_range(&self, height: usize) -> (usize, usize) {
        self.cursor.viewport_range(height)
    }
}
