use std::{collections::HashSet, fmt, iter::FromIterator};

use crate::core::listbox::Listbox;

mod render;
pub use render::Renderer;

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
}
