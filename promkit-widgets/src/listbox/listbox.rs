use std::fmt;

use promkit_core::grapheme::StyledGraphemes;

/// A `Listbox` struct that encapsulates a list of strings,
/// allowing for navigation and manipulation through a selected position.
/// It supports basic operations
/// such as moving the selection forward and backward,
/// retrieving the current item,
/// and initializing from an iterator of displayable items.
#[derive(Clone, Default)]
pub struct Listbox {
    items: Vec<StyledGraphemes>,
    selected: Option<usize>,
}

impl<E: fmt::Display, I: IntoIterator<Item = E>> From<I> for Listbox {
    fn from(items: I) -> Self {
        let items = items
            .into_iter()
            .map(|e| StyledGraphemes::from(format!("{}", e)))
            .collect::<Vec<_>>();
        let selected = (!items.is_empty()).then_some(0);

        Self { items, selected }
    }
}

impl Listbox {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push_string(&mut self, item: String) {
        self.items.push(StyledGraphemes::from(item));
        if self.selected.is_none() {
            self.selected = Some(0);
        }
    }

    /// Creates a new `Listbox` from a vector of `StyledGraphemes`.
    pub fn from_styled_graphemes(items: Vec<StyledGraphemes>) -> Self {
        let selected = (!items.is_empty()).then_some(0);
        Self { items, selected }
    }

    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<StyledGraphemes> {
        &self.items
    }

    /// Returns the selected position, defaulting to zero when empty.
    ///
    /// Use [`Self::selected`] when the empty state needs to be distinguished.
    pub fn position(&self) -> usize {
        self.selected.unwrap_or_default()
    }

    /// Returns the selected position, or `None` when the listbox is empty.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Retrieves the item at the current cursor position as a `String`.
    /// If the cursor is at a position without an item,
    /// returns an empty `String`.
    pub fn get(&self) -> StyledGraphemes {
        self.selected
            .and_then(|position| self.items.get(position))
            .unwrap_or(&StyledGraphemes::default())
            .clone()
    }

    /// Moves the cursor backward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        let Some(position) = self.selected.filter(|position| *position > 0) else {
            return false;
        };
        self.selected = Some(position - 1);
        true
    }

    /// Moves the cursor forward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        let Some(position) = self
            .selected
            .filter(|position| position.saturating_add(1) < self.items.len())
        else {
            return false;
        };
        self.selected = Some(position + 1);
        true
    }

    /// Moves the cursor to an item by index.
    pub fn move_to(&mut self, position: usize) -> bool {
        if position < self.items.len() {
            self.selected = Some(position);
            true
        } else {
            false
        }
    }

    /// Moves the cursor to the head (beginning) of the listbox.
    pub fn move_to_head(&mut self) {
        self.selected = (!self.items.is_empty()).then_some(0);
    }

    /// Moves the cursor to the tail of the listbox.
    pub fn move_to_tail(&mut self) {
        self.selected = self.items.len().checked_sub(1);
    }

    pub fn is_tail(&self) -> bool {
        self.selected.is_some() && self.selected == self.items.len().checked_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use promkit_core::grapheme::StyledGraphemes;

    use super::Listbox;

    #[test]
    fn default_is_empty() {
        let listbox = Listbox::default();
        assert!(listbox.is_empty());
        assert_eq!(listbox.len(), 0);
        assert_eq!(listbox.selected(), None);
        assert!(!listbox.is_tail());
    }

    #[test]
    fn pushing_first_item_selects_it() {
        let mut listbox = Listbox::default();
        listbox.push_string("first".into());

        assert_eq!(listbox.selected(), Some(0));
        assert_eq!(listbox.get(), StyledGraphemes::from("first"));
    }

    #[test]
    fn navigation_stops_at_the_list_boundaries() {
        let mut listbox = Listbox::from(["first", "second"]);

        assert_eq!(listbox.selected(), Some(0));
        assert!(!listbox.backward());
        assert!(listbox.forward());
        assert_eq!(listbox.selected(), Some(1));
        assert!(!listbox.forward());

        assert!(listbox.move_to(0));
        assert_eq!(listbox.selected(), Some(0));
        assert!(!listbox.move_to(2));
        assert_eq!(listbox.selected(), Some(0));

        listbox.move_to_head();
        assert_eq!(listbox.selected(), Some(0));
        listbox.move_to_tail();
        assert_eq!(listbox.selected(), Some(1));
        assert!(listbox.is_tail());
    }
}
