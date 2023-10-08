use std::iter::FromIterator;

use radix_trie::{Trie, TrieCommon};

/// Store the suggestions for completion.
#[derive(Default)]
pub struct Suggest(Trie<String, usize>);

impl<T: AsRef<str>> FromIterator<T> for Suggest {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let kvs = iter.into_iter().map(|e| (e.as_ref().to_string(), 1));
        Suggest(Trie::from_iter(kvs))
    }
}

impl Suggest {
    pub fn insert<T: AsRef<str>>(&mut self, item: T) {
        self.0.insert(item.as_ref().to_string(), 1);
    }

    /// Search the most appropriate item from the suggestions.
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
