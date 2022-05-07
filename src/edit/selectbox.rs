use std::ops::{Deref, DerefMut};

use crate::{
    edit::{Cursor, Editor, Register},
    grapheme::Graphemes,
};

/// Store the candidates to choose the items from.
#[derive(Debug, Clone, Default)]
pub struct SelectBox(Editor<Vec<Graphemes>>);

impl Deref for SelectBox {
    type Target = Editor<Vec<Graphemes>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SelectBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Into<String>> Register<T> for SelectBox {
    fn register(&mut self, item: T) {
        self.data.push(Graphemes::from(item.into()))
    }
}

impl SelectBox {
    pub fn get_with(&self, i: usize) -> Option<&Graphemes> {
        self.data.get(i)
    }

    pub fn get(&self) -> Graphemes {
        self.data
            .get(self.pos())
            .map(|v| v.to_owned())
            .unwrap_or_default()
    }
}

#[test]
fn prev() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(!b.prev());
    b.idx.set(1);
    assert!(b.prev());
}

#[test]
fn next() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(b.next());
    b.idx.set(b.data.len() - 1);
    assert!(!b.next());
}
