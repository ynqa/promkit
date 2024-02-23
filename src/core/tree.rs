mod node;

use crate::core::cursor::Cursor;

pub use node::{Kind, Node, Path};
mod render;
pub use render::Renderer;

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
            cursor: Cursor::new(root.flatten_visibles()),
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
        let kind = self.cursor.contents().get(self.position()).unwrap();
        match kind {
            Kind::Folded { id, path } | Kind::Unfolded { id, path } => {
                let mut ret = self.root.get_waypoints(path);
                ret.push(id.to_string());
                ret
            }
        }
    }

    /// Toggles the state of the current node and updates the cursor position accordingly.
    pub fn toggle(&mut self) {
        if let Some(Kind::Folded { path, .. }) | Some(Kind::Unfolded { path, .. }) =
            self.cursor.contents().get(self.position())
        {
            self.root.toggle(path);
            self.cursor = Cursor::new_with_position(self.root.flatten_visibles(), self.position());
        }
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
