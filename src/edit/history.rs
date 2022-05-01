use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use crate::{
    edit::{Cursor, Editor, Register},
    grapheme::Graphemes,
};

// TODO: change the limitations configurable for storing the historical inputs.
// TODO: add a config to choose whether the duplications are accepted.
/// New type of editor to store the histroy of the user inputs.
#[derive(Debug, Clone)]
pub struct History(Editor<Vec<Graphemes>>);

impl Deref for History {
    type Target = Editor<Vec<Graphemes>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for History {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for History {
    fn default() -> Self {
        History(Editor::<Vec<Graphemes>> {
            data: vec![Graphemes::default()],
            idx: Cell::new(0),
        })
    }
}

impl Register<Graphemes> for History {
    /// Register an item.
    ///
    /// # NOTE
    ///
    /// Register the items to the history with the following steps:
    ///
    /// 1. items = [""]
    /// 1. Input: "abc"
    /// 1. items = ["abc", ""]
    /// 1. Input "xyz"
    /// 1. items = ["abc", "xyz", ""]
    fn register(&mut self, item: Graphemes) {
        if self.data.is_empty() {
            self.data.push(item)
        } else {
            if !self.exists(&item) {
                let tail_idx = self.data.len() - 1;
                self.data.insert(tail_idx, item);
            }
            let tail_idx = self.data.len() - 1;
            self.move_to(tail_idx);
        }
    }
}

impl History {
    pub fn get(&self) -> Graphemes {
        self.data
            .get(self.pos())
            .map(|v| v.to_owned())
            .unwrap_or_default()
    }

    /// Check whether the item exists or not.
    fn exists(&self, item: &Graphemes) -> bool {
        self.data.iter().any(|i| i == item)
    }

    /// Move the cursor to the given position in the history.
    fn move_to(&self, idx: usize) -> bool {
        if idx < self.data.len() {
            self.idx.set(idx);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::{Graphemes, History, Register};

    use crate::edit::Cursor;

    #[test]
    fn register() {
        let mut h = History::default();
        h.register(Graphemes::from("line"));
        assert_eq!(h.pos(), 1);
        assert_eq!(h.get(), Graphemes::default());
    }

    #[test]
    fn exists() {
        let mut h = History::default();
        h.register(Graphemes::from("existed"));
        assert!(h.exists(&Graphemes::from("existed")));
        assert!(!h.exists(&Graphemes::from("not_found")));
    }

    #[test]
    fn move_to() {
        let mut h = History::default();
        h.register_all(vec![
            Graphemes::from("a"),
            Graphemes::from("b"),
            Graphemes::from("c"),
        ]);
        assert!(h.move_to(h.data.len() - 1));
        assert!(h.move_to(0));
        let idx_over_len = h.data.len() + 20;
        assert!(!h.move_to(idx_over_len));
    }
}
