use std::{fmt, iter::FromIterator};

use crate::core::cursor::Cursor;

mod render;
pub use render::Renderer;

/// A `Listbox` struct that encapsulates a list of strings,
/// allowing for navigation and manipulation through a cursor.
/// It supports basic operations
/// such as moving the cursor forward and backward,
/// retrieving the current item,
/// and initializing from an iterator of displayable items.
#[derive(Clone)]
pub struct Listbox(Cursor<Vec<String>>);

impl<T: fmt::Display> FromIterator<T> for Listbox {
    /// Creates a `Listbox` from an iterator of items
    /// that implement the `Display` trait.
    /// Each item is converted to a `String`
    /// and collected into a `Vec<String>`.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Cursor::new(
            iter.into_iter().map(|e| format!("{}", e)).collect(),
            0,
            false,
        ))
    }
}

impl Listbox {
    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<String> {
        self.0.contents()
    }

    /// Returns the current position of the cursor within the listbox.
    pub fn position(&self) -> usize {
        self.0.position()
    }

    /// Retrieves the item at the current cursor position as a `String`.
    /// If the cursor is at a position without an item,
    /// returns an empty `String`.
    pub fn get(&self) -> String {
        self.items()
            .get(self.position())
            .unwrap_or(&String::new())
            .to_string()
    }

    /// Moves the cursor backward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.0.backward()
    }

    /// Moves the cursor forward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.0.forward()
    }

    /// Moves the cursor to the head (beginning) of the listbox.
    pub fn move_to_head(&mut self) {
        self.0.move_to_head()
    }

    /// Moves the cursor to the tail of the listbox.
    pub fn move_to_tail(&mut self) {
        self.0.move_to_tail()
    }
}
