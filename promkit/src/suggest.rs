use std::{fmt, iter::FromIterator};

use radix_trie::{Trie, TrieCommon};

/// A structure to store and manage suggestions for autocompletion.
/// It utilizes a trie for efficient storage and retrieval of suggestions.
/// This allows for quick lookup of suggestions based on a given prefix,
/// making it suitable for use in text editors or command line interfaces
/// where autocompletion features are desired.
#[derive(Clone)]
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
        Suggest(Trie::from_iter(
            iter.into_iter().map(|e| (format!("{}", e), 1)),
        ))
    }
}

impl Suggest {
    pub fn prefix_search<T: AsRef<str>>(&self, query: T) -> Option<Vec<String>> {
        self.0
            .get_raw_descendant(query.as_ref())
            .map(|subtrie| subtrie.iter().map(|item| item.0.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod suggest {
        use super::*;

        #[test]
        fn test() {
            let mut trie = Trie::new();
            trie.insert("apple".to_string(), 1);
            trie.insert("applet".to_string(), 1);
            trie.insert("application".to_string(), 1);
            trie.insert("banana".to_string(), 1);

            let suggest = Suggest(trie);
            let ret = suggest.prefix_search("app").unwrap();
            let expected: Vec<String> = vec!["apple", "applet", "application"]
                .into_iter()
                .map(String::from)
                .collect();

            assert_eq!(ret, expected);
        }
    }
}
