use crate::core::cursor::Cursor;

mod node;
pub use node::{JsonPath, JsonPathSegment, Kind, Node};

pub struct Json {
    root: Node,
    cursor: Cursor<Vec<Kind>>,
}

impl Json {
    pub fn new(root: Node) -> Self {
        Self {
            root: root.clone(),
            cursor: Cursor::new(root.flatten_visibles()),
        }
    }

    pub fn kinds(&self) -> Vec<Kind> {
        self.cursor.contents().clone()
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn get(&self) -> JsonPath {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let binding = vec![];
        let path = match kind {
            Kind::ArrayEntry { path, .. } => path,
            Kind::MapEntry { path, .. } => path,
            _ => &binding,
        };

        path.clone()
    }

    pub fn toggle(&mut self) {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let binding = vec![];
        let route = match kind {
            Kind::ArrayStart { path, .. } => path,
            Kind::ArrayFolded { path, .. } => path,
            Kind::MapStart { path, .. } => path,
            Kind::MapFolded { path, .. } => path,
            _ => &binding,
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
