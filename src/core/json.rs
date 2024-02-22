use crate::core::cursor::Cursor;

mod node;
pub use node::{Index, Kind, Node};

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

    pub fn get(&self) -> Vec<Index> {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let binding = vec![];
        let route = match kind {
            Kind::ArrayEntry {
                v: _,
                index,
                is_last: _,
            } => index,
            Kind::MapEntry {
                kv: _,
                index,
                is_last: _,
            } => index,
            _ => &binding,
        };

        route.clone()
    }

    pub fn toggle(&mut self) {
        let kind = self.cursor.contents().get(self.position()).unwrap();
        let binding = vec![];
        let route = match kind {
            Kind::ArrayStart { key: _, index } => index,
            Kind::ArrayFolded {
                key: _,
                index,
                is_last: _,
            } => index,
            Kind::MapStart { key: _, index } => index,
            Kind::MapFolded {
                key: _,
                index,
                is_last: _,
            } => index,
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
