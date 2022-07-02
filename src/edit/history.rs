use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use crate::{
    edit::{Cursor, Editor, Register, SelectBox},
    grapheme::Graphemes,
};

/// Store the histroy of the past user inputs.
#[derive(Debug, Clone)]
pub struct History {
    selectbox: SelectBox,

    pub limit_len: Option<usize>,
}

impl Deref for History {
    type Target = SelectBox;
    fn deref(&self) -> &Self::Target {
        &self.selectbox
    }
}

impl DerefMut for History {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selectbox
    }
}

impl Default for History {
    fn default() -> Self {
        History {
            selectbox: SelectBox(Editor::<Vec<Graphemes>> {
                data: vec![Graphemes::default()],
                idx: Cell::new(0),
            }),
            limit_len: None,
        }
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
        if self.selectbox.data.is_empty() {
            self.selectbox.data.push(item)
        } else {
            if !self.exists(&item) {
                let tail_idx = self.selectbox.data.len() - 1;
                self.selectbox.data.insert(tail_idx, item);
                // Oldest one of history is removed
                // when the history is filled.
                if let Some(limit) = self.limit_len {
                    // Plus 1 considers the current input.
                    if limit + 1 < self.selectbox.data.len() {
                        self.selectbox.data.remove(0);
                    }
                }
            }
            let tail_idx = self.selectbox.data.len() - 1;
            self.move_to(tail_idx);
        }
    }
}

impl History {
    pub fn get(&self) -> Graphemes {
        self.selectbox
            .data
            .get(self.selectbox.pos())
            .map(|v| v.to_owned())
            .unwrap_or_default()
    }

    /// Check whether the item exists or not.
    fn exists(&self, item: &Graphemes) -> bool {
        self.selectbox.data.iter().any(|i| i == item)
    }

    /// Move the cursor to the given position in the history.
    fn move_to(&self, idx: usize) -> bool {
        if idx < self.selectbox.data.len() {
            self.selectbox.idx.set(idx);
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
        assert_eq!(h.selectbox.pos(), 1);
        assert_eq!(h.get(), Graphemes::default());
    }

    #[test]
    fn register_with_limit_len() {
        let mut h = History {
            limit_len: Some(3),
            ..Default::default()
        };
        h.register_all(vec![
            Graphemes::from("a"),
            Graphemes::from("b"),
            Graphemes::from("c"),
        ]);
        h.register(Graphemes::from("d"));
        assert_eq!(
            vec![
                Graphemes::from("b"),
                Graphemes::from("c"),
                Graphemes::from("d"),
                Graphemes::default(),
            ],
            h.data,
        )
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
        assert!(h.move_to(h.selectbox.data.len() - 1));
        assert!(h.move_to(0));
        let idx_over_len = h.selectbox.data.len() + 20;
        assert!(!h.move_to(idx_over_len));
    }
}
