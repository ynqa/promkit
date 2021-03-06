use std::cell::Cell;

use crate::{grapheme::Graphemes, register::Register};

/// Store the candidates to choose the items from.
#[derive(Debug, Clone, Default)]
pub struct SelectBox {
    pub data: Vec<Graphemes>,
    pub position: Cell<usize>,
}

impl<T: Into<String>> Register<T> for SelectBox {
    fn register(&mut self, item: T) {
        self.data.push(Graphemes::from(item.into()))
    }
}

impl SelectBox {
    pub fn position(&self) -> usize {
        self.position.get()
    }

    pub fn prev(&self) -> bool {
        if 0 < self.position.get() {
            self.position.set(self.position.get() - 1);
            return true;
        }
        false
    }

    pub fn next(&self) -> bool {
        if !self.data.is_empty() && self.position.get() < self.data.len() - 1 {
            self.position.set(self.position.get() + 1);
            return true;
        }
        false
    }

    pub fn to_head(&self) {
        self.position.set(0)
    }

    pub fn to_tail(&self) {
        self.position.set(self.data.len() - 1)
    }

    pub fn get_with_index(&self, i: usize) -> Graphemes {
        self.data.get(i).map(|v| v.to_owned()).unwrap_or_default()
    }

    pub fn get(&self) -> Graphemes {
        self.get_with_index(self.position())
    }
}

#[test]
fn prev() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(!b.prev());
    b.position.set(1);
    assert!(b.prev());
}

#[test]
fn next() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(b.next());
    b.position.set(b.data.len() - 1);
    assert!(!b.next());
}
