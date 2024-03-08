use crate::core::cursor::Cursor;

mod node;
pub use node::{JsonNode, JsonPath, JsonPathSegment, JsonSyntaxKind};
mod render;
pub use render::{Renderer, Theme};
pub mod bundle;
pub use bundle::JsonBundle;
pub mod keymap;

/// A `Json` structure that manages a JSON document as a tree of nodes.
/// It utilizes a cursor to navigate and manipulate the nodes within the JSON tree.
#[derive(Clone)]
pub struct Json {
    root: JsonNode,
    cursor: Cursor<Vec<JsonSyntaxKind>>,
}

impl Json {
    /// Creates a new `Json` with a given root node.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node of the JSON tree.
    pub fn new(root: JsonNode) -> Self {
        Self {
            root: root.clone(),
            cursor: Cursor::new(root.flatten_visibles()),
        }
    }

    /// Returns a reference to the root node of the JSON tree.
    pub fn root(&self) -> &JsonNode {
        &self.root
    }

    /// Returns a vector of all `JsonSyntaxKind` in the tree, representing the visible nodes.
    pub fn kinds(&self) -> Vec<JsonSyntaxKind> {
        self.cursor.contents().clone()
    }

    /// Retrieves the `JsonPath` of the current node pointed by the cursor.
    pub fn path_from_root(&self) -> JsonPath {
        let kind = self.cursor.contents()[self.position()].clone();
        let binding = vec![];
        let path = match kind {
            JsonSyntaxKind::ArrayEntry { path, .. } => path,
            JsonSyntaxKind::MapEntry { path, .. } => path,
            _ => binding,
        };

        path.clone()
    }

    /// Toggles the state of the current node (e.g., from expanded to folded) and updates the cursor position accordingly.
    pub fn toggle(&mut self) {
        let kind = self.cursor.contents()[self.position()].clone();
        let route = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => return,
        };

        self.root.toggle(&route);
        self.cursor = Cursor::new_with_position(self.root.flatten_visibles(), self.position());
    }

    /// Returns the current position of the cursor within the JSON tree.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    /// Moves the cursor backward in the JSON tree, if possible.
    ///
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    /// Moves the cursor forward in the JSON tree, if possible.
    ///
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the cursor to the head of the JSON tree.
    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }

    /// Moves the cursor to the tail of the JSON tree.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
    }
}
