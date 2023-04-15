use std::fmt;
use std::ops::{Deref, DerefMut};

use radix_trie::{Trie, TrieCommon};

use crate::{grapheme::Graphemes, register::Register};

/// Store the suggestions for completion.
#[derive(Clone, Debug, Default)]
pub struct Suggest(pub Trie<Graphemes, usize>);

impl Deref for Suggest {
    type Target = Trie<Graphemes, usize>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Suggest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: fmt::Display> Register<T> for Suggest {
    fn register(&mut self, item: T) {
        self.insert(Graphemes::from(format!("{}", item)), 1);
    }
}

impl Suggest {
    /// Search the most appropriate item from the suggestions.
    pub fn search(&self, query: &Graphemes) -> Option<Graphemes> {
        match self.get_raw_descendant(query) {
            Some(subtrie) => subtrie.iter().next().map(|item| item.0.clone()),
            None => None,
        }
    }
}

#[test]
fn register() {
    let mut s = Suggest::default();
    s.register_all(vec!["abc", "abd", "abxyz"]);
    assert_eq!(s.len(), 3);
}

#[test]
fn search() {
    let mut s = Suggest::default();
    s.register_all(vec!["abc", "abd", "abxyz"]);
    assert_eq!(s.search(&Graphemes::from("")), Some(Graphemes::from("abc")));
    assert_eq!(s.search(&Graphemes::from("x")), None);
    assert_eq!(
        s.search(&Graphemes::from("ab")),
        Some(Graphemes::from("abc"))
    );
    assert_eq!(
        s.search(&Graphemes::from("abd")),
        Some(Graphemes::from("abd"))
    );
}
