use std::fmt;

use promkit_core::grapheme::StyledGraphemes;

use crate::cursor::Cursor;

mod state;
pub use state::State;

/// A `Listbox` struct that encapsulates a list of strings,
/// allowing for navigation and manipulation through a cursor.
/// It supports basic operations
/// such as moving the cursor forward and backward,
/// retrieving the current item,
/// and initializing from an iterator of displayable items.
#[derive(Clone)]
pub struct Listbox(Cursor<Vec<StyledGraphemes>>);

impl Default for Listbox {
    fn default() -> Self {
        Self(Cursor::new(vec![StyledGraphemes::default()], 0, false))
    }
}

impl Listbox {
    /// Creates a new `Listbox` from a vector of `fmt::Display`.
    pub fn from_displayable<E: fmt::Display, I: IntoIterator<Item = E>>(items: I) -> Self {
        Self(Cursor::new(
            items
                .into_iter()
                .map(|e| StyledGraphemes::from(format!("{}", e)))
                .collect(),
            0,
            false,
        ))
    }

    pub fn len(&self) -> usize {
        self.0.contents().len()
    }

    pub fn push_string(&mut self, item: String) {
        self.0.contents_mut().push(StyledGraphemes::from(item));
    }

    /// Creates a new `Listbox` from a vector of `StyledGraphemes`.
    pub fn from_styled_graphemes(items: Vec<StyledGraphemes>) -> Self {
        Self(Cursor::new(items, 0, false))
    }

    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<StyledGraphemes> {
        self.0.contents()
    }

    /// Returns the current position of the cursor within the listbox.
    pub fn position(&self) -> usize {
        self.0.position()
    }

    /// Retrieves the item at the current cursor position as a `String`.
    /// If the cursor is at a position without an item,
    /// returns an empty `String`.
    pub fn get(&self) -> StyledGraphemes {
        self.items()
            .get(self.position())
            .unwrap_or(&StyledGraphemes::default())
            .clone()
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

    pub fn is_tail(&self) -> bool {
        self.0.is_tail()
    }
}
