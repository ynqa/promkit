use std::{collections::HashSet, fmt, iter::FromIterator};

use crate::core::listbox::Listbox;

mod state;
pub use state::State;

/// A `Checkbox` struct that encapsulates a listbox
/// for item selection and a set of picked (selected) indices.
/// It allows for multiple selections,
/// toggling the selection state of items,
/// and navigating through the items.
#[derive(Clone)]
pub struct Checkbox {
    listbox: Listbox,
    picked: HashSet<usize>,
}

impl<T: fmt::Display> FromIterator<T> for Checkbox {
    /// Creates a `Checkbox` from an iterator of items
    /// that implement the `Display` trait.
    /// Each item is added to the listbox,
    /// and the set of picked indices is initialized as empty.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            listbox: Listbox::from_iter(iter),
            picked: HashSet::new(),
        }
    }
}

impl Checkbox {
    /// Creates a `Checkbox` from an iterator of tuples where the first element
    /// implements the `Display` trait and the second element is a bool indicating
    /// if the item is picked (selected).
    /// Each item is added to the listbox, and the set of picked indices is
    /// initialized based on the bool values.
    pub fn new_with_checked<T: fmt::Display, I: IntoIterator<Item = (T, bool)>>(iter: I) -> Self {
        let (listbox, picked): (Vec<_>, Vec<_>) = iter
            .into_iter()
            .enumerate()
            .map(|(index, (item, is_picked))| ((index, item), is_picked))
            .unzip(); // `unzip` を使用して、アイテムと選択状態を分けます。

        let listbox_items = listbox
            .into_iter()
            .map(|(_, item)| item)
            .collect::<Vec<_>>();
        let picked_indices = picked
            .into_iter()
            .enumerate()
            .filter_map(
                |(index, is_picked)| {
                    if is_picked {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect::<HashSet<usize>>();

        Self {
            listbox: Listbox::from_iter(listbox_items),
            picked: picked_indices,
        }
    }

    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<String> {
        self.listbox.items()
    }

    /// Returns the current position of the cursor within the listbox.
    pub fn position(&self) -> usize {
        self.listbox.position()
    }

    /// Returns a reference to the set of picked (selected) indices.
    pub fn picked_indexes(&self) -> &HashSet<usize> {
        &self.picked
    }

    /// Retrieves the items at the picked (selected) indices as a vector of strings.
    pub fn get(&self) -> Vec<String> {
        self.picked
            .iter()
            .fold(Vec::<String>::new(), |mut ret, idx| {
                ret.push(self.listbox.items().get(*idx).unwrap().to_owned());
                ret
            })
    }

    /// Toggles the selection state of the item at the current cursor position within the listbox.
    pub fn toggle(&mut self) {
        if self.picked.contains(&self.listbox.position()) {
            self.picked.remove(&self.listbox.position());
        } else {
            self.picked.insert(self.listbox.position());
        }
    }

    /// Moves the cursor backward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.listbox.backward()
    }

    /// Moves the cursor forward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.listbox.forward()
    }

    /// Moves the cursor to the head (beginning) of the listbox.
    pub fn move_to_head(&mut self) {
        self.listbox.move_to_head()
    }

    /// Moves the cursor to the tail of the listbox.
    pub fn move_to_tail(&mut self) {
        self.listbox.move_to_tail()
    }

    pub fn viewport_range(&self, height: usize) -> (usize, usize) {
        self.listbox.viewport_range(height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new_with_checked {
        use super::*;

        #[test]
        fn test() {
            // Prepare a list of items with their checked status
            let items = vec![
                (String::from("1"), true),
                (String::from("2"), false),
                (String::from("3"), true),
            ];

            // Create a Checkbox using `new_with_checked`
            let checkbox = Checkbox::new_with_checked(items);

            // Verify the items in the listbox
            assert_eq!(
                checkbox.items(),
                &vec!["1".to_string(), "2".to_string(), "3".to_string()]
            );

            // Verify the picked (selected) indices
            let expected_picked_indexes: HashSet<usize> = [0, 2].iter().cloned().collect();
            assert_eq!(checkbox.picked_indexes(), &expected_picked_indexes);
        }
    }
}
