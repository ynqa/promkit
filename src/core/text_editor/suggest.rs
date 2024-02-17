use std::{fmt, iter::FromIterator};

use radix_trie::{Trie, TrieCommon};

/// A structure to store and manage suggestions for autocompletion.
/// It utilizes a trie for efficient storage and retrieval of suggestions.
/// This allows for quick lookup of suggestions based on a given prefix,
/// making it suitable for use in text editors or command line interfaces
/// where autocompletion features are desired.
#[derive(Clone, Default)]
pub struct Suggest(Trie<String, usize>);

impl<T: fmt::Display> FromIterator<T> for Suggest {
    /// Constructs a `Suggest` instance from an iterator of displayable items.
    /// Each item is inserted into the trie with a default value to facilitate
    /// quick prefix-based searches.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator over items that implement the `Display` trait.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let kvs = iter.into_iter().map(|e| (format!("{}", e), 1));
        Suggest(Trie::from_iter(kvs))
    }
}

impl Suggest {
    /// Inserts a new suggestion into the trie.
    /// If the suggestion already exists, this method has no effect.
    ///
    /// # Arguments
    ///
    /// * `item` - The suggestion to insert.
    pub fn insert<T: AsRef<str>>(&mut self, item: T) {
        self.0.insert(item.as_ref().to_string(), 1);
    }

    /// Searches for the most appropriate suggestion based on the given query.
    /// Returns the closest match if found, or `None` if no suitable suggestion exists.
    ///
    /// # Arguments
    ///
    /// * `query` - The query string to search for.
    pub fn search<T: AsRef<str>>(&self, query: T) -> Option<&str> {
        match self.0.get_raw_descendant(query.as_ref()) {
            Some(subtrie) => subtrie.iter().next().map(|item| item.0.as_str()),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    mod insert {
        use super::super::*;

        #[test]
        fn test() {
            let mut s = Suggest::default();
            s.insert("abc");
            s.insert("abd");
            s.insert("abxyz");
            assert_eq!(s.0.len(), 3);
        }
    }

    mod search {
        use super::super::*;

        #[test]
        fn test() {
            let s = Suggest::from_iter(["abc", "abd", "abxyz"]);
            assert_eq!(s.search(""), Some("abc"));
            assert_eq!(s.search("x"), None);
            assert_eq!(s.search("a"), Some("abc"));
            assert_eq!(s.search("ab"), Some("abc"));
            assert_eq!(s.search("abd"), Some("abd"));
        }
    }
}
