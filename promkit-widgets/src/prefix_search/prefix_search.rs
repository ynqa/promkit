use std::{fmt, iter::FromIterator, sync::Arc};

use radix_trie::{Trie, TrieCommon};

/// Prefix-search candidates backed directly by a radix trie.
#[derive(Clone)]
pub struct PrefixSearch {
    candidates: Arc<[String]>,
    index: Trie<String, usize>,
}

impl Default for PrefixSearch {
    fn default() -> Self {
        Self {
            candidates: Arc::default(),
            index: Trie::new(),
        }
    }
}

impl<T: fmt::Display> FromIterator<T> for PrefixSearch {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let candidates = iter
            .into_iter()
            .map(|item| item.to_string())
            .collect::<Arc<[_]>>();
        let index = Trie::from_iter(
            candidates
                .iter()
                .enumerate()
                .map(|(index, candidate)| (candidate.clone(), index)),
        );

        Self { candidates, index }
    }
}

impl PrefixSearch {
    /// Creates an independently selectable snapshot of candidates matching `query`.
    pub fn query(&self, query: impl AsRef<str>) -> PrefixSearchResult {
        let candidates = self
            .index
            .get_raw_descendant(query.as_ref())
            .into_iter()
            .flat_map(|subtrie| subtrie.iter().map(|(_, index)| *index))
            .collect::<Vec<_>>();
        let selected = (!candidates.is_empty()).then_some(0);

        PrefixSearchResult {
            source: Arc::clone(&self.candidates),
            candidates,
            selected,
        }
    }
}

/// A selectable snapshot produced by a prefix query.
#[derive(Clone, Default)]
pub struct PrefixSearchResult {
    source: Arc<[String]>,
    candidates: Vec<usize>,
    selected: Option<usize>,
}

impl PrefixSearchResult {
    /// Clears the snapshot and its selection.
    pub fn clear(&mut self) {
        self.candidates.clear();
        self.selected = None;
    }

    /// Returns the candidates in this snapshot.
    pub fn candidates(&self) -> impl Iterator<Item = &str> {
        self.candidates
            .iter()
            .filter_map(|index| self.source.get(*index))
            .map(String::as_str)
    }

    /// Returns the candidate at `index`.
    pub fn candidate_at(&self, index: usize) -> Option<&str> {
        self.candidates
            .get(index)
            .and_then(|index| self.source.get(*index))
            .map(String::as_str)
    }

    /// Returns the selected candidate index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the selected candidate.
    pub fn get(&self) -> Option<&str> {
        self.selected
            .and_then(|selected| self.candidate_at(selected))
    }

    /// Returns whether the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Moves the selection to the previous candidate.
    pub fn backward(&mut self) -> bool {
        let Some(selected) = self.selected.filter(|selected| *selected > 0) else {
            return false;
        };
        self.selected = Some(selected - 1);
        true
    }

    /// Moves the selection to the next candidate.
    pub fn forward(&mut self) -> bool {
        let Some(selected) = self
            .selected
            .filter(|selected| selected.saturating_add(1) < self.candidates.len())
        else {
            return false;
        };
        self.selected = Some(selected + 1);
        true
    }

    /// Moves the selection to a candidate by index.
    pub fn move_to(&mut self, index: usize) -> bool {
        if index < self.candidates.len() {
            self.selected = Some(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PrefixSearch, PrefixSearchResult};

    fn prefix_search() -> PrefixSearch {
        ["apple", "applet", "application", "banana"]
            .into_iter()
            .collect()
    }

    mod prefix_search {
        use super::*;

        mod query {
            use super::*;

            #[test]
            fn returns_an_independently_selectable_snapshot() {
                let prefix_search = prefix_search();

                let mut application = prefix_search.query("app");
                let banana = prefix_search.query("ban");
                application.forward();

                assert_eq!(
                    application.candidates().collect::<Vec<_>>(),
                    vec!["apple", "applet", "application"]
                );
                assert_eq!(application.get(), Some("applet"));
                assert_eq!(banana.candidates().collect::<Vec<_>>(), vec!["banana"]);
                assert_eq!(banana.get(), Some("banana"));
            }

            #[test]
            fn returns_an_empty_snapshot_for_a_missing_prefix() {
                let result = prefix_search().query("orange");

                assert!(result.is_empty());
                assert_eq!(result.selected(), None);
                assert_eq!(result.get(), None);
            }
        }
    }

    mod prefix_search_result {
        use super::*;

        fn result() -> PrefixSearchResult {
            prefix_search().query("app")
        }

        mod backward {
            use super::*;

            #[test]
            fn stops_at_the_first_candidate() {
                let mut result = result();

                assert!(!result.backward());
                assert_eq!(result.get(), Some("apple"));
            }
        }

        mod forward {
            use super::*;

            #[test]
            fn stops_at_the_last_candidate() {
                let mut result = result();

                assert!(result.forward());
                assert_eq!(result.get(), Some("applet"));
                assert!(result.forward());
                assert_eq!(result.get(), Some("application"));
                assert!(!result.forward());
            }
        }

        mod move_to {
            use super::*;

            #[test]
            fn rejects_an_out_of_bounds_index() {
                let mut result = result();
                result.move_to(2);

                assert!(!result.move_to(3));
                assert_eq!(result.selected(), Some(2));
            }

            #[test]
            fn selects_the_given_candidate() {
                let mut result = result();
                result.move_to(2);

                assert!(result.move_to(0));
                assert_eq!(result.get(), Some("apple"));
            }
        }

        mod clear {
            use super::*;

            #[test]
            fn removes_candidates_and_selection_from_the_snapshot() {
                let mut result = result();

                result.clear();

                assert!(result.is_empty());
                assert_eq!(result.selected(), None);
                assert_eq!(result.get(), None);
            }
        }
    }
}
