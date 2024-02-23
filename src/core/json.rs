use crate::core::cursor::Cursor;

mod node;
pub use node::{JsonNode, JsonPath, JsonPathSegment, JsonSyntaxKind};
mod render;
pub use render::Renderer;

#[derive(Clone)]
pub struct Json {
    root: JsonNode,
    cursor: Cursor<Vec<JsonSyntaxKind>>,
}

impl Json {
    pub fn new(root: JsonNode) -> Self {
        Self {
            root: root.clone(),
            cursor: Cursor::new(root.flatten_visibles()),
        }
    }

    pub fn kinds(&self) -> Vec<JsonSyntaxKind> {
        self.cursor.contents().clone()
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn get(&self) -> JsonPath {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let binding = vec![];
        let path = match kind {
            JsonSyntaxKind::ArrayEntry { path, .. } => path,
            JsonSyntaxKind::MapEntry { path, .. } => path,
            _ => &binding,
        };

        path.clone()
    }

    pub fn toggle(&mut self) {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let route = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => return,
        };

        self.root.toggle(route);
        self.cursor = Cursor::new_with_position(self.root.flatten_visibles(), self.position());
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
