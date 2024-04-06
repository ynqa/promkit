use crate::core::cursor::CompositeCursor;

mod node;
pub use node::{JsonNode, JsonPath, JsonPathSegment, JsonSyntaxKind};
mod render;
pub use render::{Renderer, Theme};

/// Represents a stream of JSON data, allowing for efficient navigation and manipulation.
///
/// `JsonStream` holds a collection of root JSON nodes and a cursor for navigating through
/// the JSON syntax kinds present in the stream. It supports operations like toggling visibility
/// of nodes, moving the cursor, and accessing specific nodes.
#[derive(Clone)]
pub struct JsonStream {
    roots: Vec<JsonNode>,
    cursor: CompositeCursor<Vec<JsonSyntaxKind>>,
}

impl JsonStream {
    pub fn new<I: IntoIterator<Item = serde_json::Value>>(iter: I, depth: Option<usize>) -> Self {
        let roots: Vec<JsonNode> = iter.into_iter().map(|v| JsonNode::new(v, depth)).collect();
        Self {
            roots: roots.clone(),
            cursor: CompositeCursor::new(roots.iter().map(|r| r.flatten_visibles()), 0),
        }
    }
}

impl JsonStream {
    /// Retrieves a reference to a root `JsonNode` by its index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the root node to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `JsonNode` if it exists, or `None` if the index is out of bounds.
    pub fn get_root(&self, index: usize) -> Option<&JsonNode> {
        self.roots.get(index)
    }

    /// Provides a reference to the vector of root `JsonNode`s.
    ///
    /// # Returns
    ///
    /// A reference to the vector containing all root nodes in the JSON stream.
    pub fn roots(&self) -> &Vec<JsonNode> {
        &self.roots
    }

    /// Flattens the visible JSON syntax kinds into a vector.
    ///
    /// This method traverses all visible nodes in the JSON stream and collects their syntax kinds into a flat vector.
    ///
    /// # Returns
    ///
    /// A `Vec<JsonSyntaxKind>` containing all visible syntax kinds from the JSON stream.
    pub fn flatten_kinds(&self) -> Vec<JsonSyntaxKind> {
        self.roots
            .iter()
            .flat_map(|root| root.flatten_visibles().into_iter())
            .collect()
    }

    /// Retrieves the current root node and its path from the root based on the cursor's position.
    ///
    /// # Returns
    ///
    /// A tuple containing the current `JsonNode` and an `Option<JsonPath>` indicating the path from the root to this node.
    pub fn current_root_and_path_from_root(&self) -> (JsonNode, Option<JsonPath>) {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();
        let kind = self.cursor.bundle()[index][inner].clone();
        (self.roots[index].clone(), kind.path().cloned())
    }

    /// Toggles the visibility of a node at the cursor's current position.
    pub fn toggle(&mut self) {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();

        let kind = self.cursor.bundle()[index][inner].clone();
        let route = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => return,
        };

        self.roots[index].toggle(&route);
        self.cursor = CompositeCursor::new(
            self.roots.iter().map(|r| r.flatten_visibles()),
            self.cursor.cross_contents_position(),
        );
    }

    /// Toggles the visibility of all nodes in the JSON tree.
    ///
    /// # Arguments
    ///
    /// * `expand` - A boolean indicating whether to expand (true) or collapse (false) all nodes.
    fn toggle_all_visibility(&mut self, expand: bool) {
        fn toggle_visibility(node: &mut JsonNode, expand: bool) {
            match node {
                JsonNode::Object {
                    children,
                    children_visible,
                } => {
                    *children_visible = expand;
                    for child in children.values_mut() {
                        toggle_visibility(child, expand);
                    }
                }
                JsonNode::Array {
                    children,
                    children_visible,
                } => {
                    *children_visible = expand;
                    for child in children.iter_mut() {
                        toggle_visibility(child, expand);
                    }
                }
                _ => {}
            }
        }

        for root in &mut self.roots {
            toggle_visibility(root, expand);
        }
        self.cursor = CompositeCursor::new(
            self.roots.iter().map(|r| r.flatten_visibles()),
            self.cursor.cross_contents_position(),
        );
    }

    /// Collapses all nodes in the JSON tree.
    pub fn collapse_all(&mut self) {
        self.toggle_all_visibility(false);
    }

    /// Expands all nodes in the JSON tree.
    pub fn expand_all(&mut self) {
        self.toggle_all_visibility(true);
    }

    /// Moves the cursor backward through the JSON stream.
    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    /// Moves the cursor forward through the JSON stream.
    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the cursor to the head of the JSON stream.
    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }

    /// Moves the cursor to the tail of the JSON stream.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
    }
}
