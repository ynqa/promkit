use std::{fmt, iter::FromIterator};

use radix_trie::{Trie, TrieCommon};

/// Prefix-search candidates backed directly by a radix trie.
#[derive(Clone)]
pub struct PrefixSearch {
    candidates: Trie<String, ()>,
    query: Option<String>,
    selected: Option<usize>,
}

impl Default for PrefixSearch {
    fn default() -> Self {
        Self {
            candidates: Trie::new(),
            query: None,
            selected: None,
        }
    }
}

impl<T: fmt::Display> FromIterator<T> for PrefixSearch {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            candidates: Trie::from_iter(iter.into_iter().map(|item| (item.to_string(), ()))),
            ..Default::default()
        }
    }
}

impl PrefixSearch {
    /// Updates the active query and selects the first matching candidate.
    ///
    /// Returns `true` when at least one candidate matches.
    pub fn search(&mut self, query: impl AsRef<str>) -> bool {
        self.query = Some(query.as_ref().to_string());
        let has_match = self.candidates().next().is_some();
        self.selected = has_match.then_some(0);
        has_match
    }

    /// Clears the active query and selection without removing candidates.
    pub fn clear(&mut self) {
        self.query = None;
        self.selected = None;
    }

    /// Returns candidates matching the active query directly from the trie.
    pub fn candidates(&self) -> impl Iterator<Item = &str> {
        self.query
            .as_deref()
            .and_then(|query| self.candidates.get_raw_descendant(query))
            .into_iter()
            .flat_map(|subtrie| subtrie.iter().map(|(candidate, _)| candidate.as_str()))
    }

    /// Returns the candidate at `index` in the current match set.
    pub fn candidate_at(&self, index: usize) -> Option<&str> {
        self.candidates().nth(index)
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

    /// Returns whether the current match set is empty.
    pub fn is_empty(&self) -> bool {
        self.candidates().next().is_none()
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
            .filter(|selected| self.candidate_at(selected.saturating_add(1)).is_some())
        else {
            return false;
        };
        self.selected = Some(selected + 1);
        true
    }

    /// Moves the selection to a candidate by index.
    pub fn move_to(&mut self, index: usize) -> bool {
        if self.candidate_at(index).is_some() {
            self.selected = Some(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrefixSearch;

    fn prefix_search() -> PrefixSearch {
        ["apple", "applet", "application", "banana"]
            .into_iter()
            .collect()
    }

    #[test]
    fn search_filters_trie_and_selects_first_match() {
        let mut prefix_search = prefix_search();

        assert!(prefix_search.search("app"));
        assert_eq!(
            prefix_search.candidates().collect::<Vec<_>>(),
            vec!["apple", "applet", "application"]
        );
        assert_eq!(prefix_search.selected(), Some(0));
        assert_eq!(prefix_search.get(), Some("apple"));

        assert!(prefix_search.search("ban"));
        assert_eq!(
            prefix_search.candidates().collect::<Vec<_>>(),
            vec!["banana"]
        );
        assert_eq!(prefix_search.selected(), Some(0));
        assert_eq!(prefix_search.get(), Some("banana"));
    }

    #[test]
    fn search_without_a_match_clears_the_selection() {
        let mut prefix_search = prefix_search();

        assert!(prefix_search.search("app"));
        assert!(!prefix_search.search("orange"));
        assert!(prefix_search.is_empty());
        assert_eq!(prefix_search.selected(), None);
        assert_eq!(prefix_search.get(), None);
    }

    #[test]
    fn navigation_stops_at_match_boundaries() {
        let mut prefix_search = prefix_search();
        prefix_search.search("app");

        assert!(!prefix_search.backward());
        assert!(prefix_search.forward());
        assert_eq!(prefix_search.get(), Some("applet"));
        assert!(prefix_search.forward());
        assert_eq!(prefix_search.get(), Some("application"));
        assert!(!prefix_search.forward());
        assert!(!prefix_search.move_to(3));
        assert_eq!(prefix_search.selected(), Some(2));
        assert!(prefix_search.move_to(0));
        assert_eq!(prefix_search.get(), Some("apple"));
    }

    #[test]
    fn clear_hides_matches_without_removing_candidates() {
        let mut prefix_search = prefix_search();
        prefix_search.search("app");

        prefix_search.clear();

        assert!(prefix_search.is_empty());
        assert_eq!(prefix_search.selected(), None);
        assert!(prefix_search.search("app"));
        assert_eq!(prefix_search.get(), Some("apple"));
    }
}
