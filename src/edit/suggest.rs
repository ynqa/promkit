use radix_trie::{Trie, TrieCommon};

use crate::{edit::Register, grapheme::Graphemes};

/// Store the suggestions for completion.
#[derive(Clone, Debug)]
pub struct Suggest {
    trie: Trie<Graphemes, usize>,

    /// Minimum length of chars to start searching.
    pub min_len_to_search: usize,
}

impl Default for Suggest {
    fn default() -> Self {
        Self {
            trie: Trie::default(),
            min_len_to_search: 1,
        }
    }
}

impl<T: Into<String>> Register<T> for Suggest {
    fn register(&mut self, item: T) {
        self.trie.insert(Graphemes::from(item.into()), 1);
    }
}

impl Suggest {
    /// Search the most appropriate item from the suggestions.
    pub fn search(&self, query: &Graphemes) -> Option<Graphemes> {
        if query.len() < self.min_len_to_search {
            return None;
        }
        match self.trie.get_raw_descendant(query) {
            Some(subtrie) => subtrie.iter().next().map(|item| item.0.clone()),
            None => None,
        }
    }
}

#[test]
fn register() {
    let mut s = Suggest::default();
    s.register_all(vec!["abc", "abd", "abxyz"]);
    assert_eq!(s.trie.len(), 3);
}

#[test]
fn search() {
    let mut s = Suggest::default();
    s.register_all(vec!["abc", "abd", "abxyz"]);
    assert_eq!(s.search(&Graphemes::from("")), None);
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
