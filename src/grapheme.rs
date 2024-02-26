use std::{
    collections::VecDeque,
    fmt::Debug,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use unicode_width::UnicodeWidthChar;

use crate::Len;

mod styled;
pub use styled::{matrixify, trim, StyledGraphemes};

/// Represents a single grapheme with its character and display width.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grapheme {
    ch: char,
    width: usize,
}

impl Grapheme {
    /// Creates a new `Grapheme` from a character, calculating its display width.
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
        }
    }

    /// Returns the display width of the grapheme.
    pub fn width(&self) -> usize {
        self.width
    }
}

/// A collection of `Grapheme` instances, stored in a `VecDeque`.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Graphemes(pub VecDeque<Grapheme>);

impl Deref for Graphemes {
    type Target = VecDeque<Grapheme>;

    /// Dereferences to the underlying `VecDeque<Grapheme>`.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Graphemes {
    /// Mutable dereference to the underlying `VecDeque<Grapheme>`.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Graphemes> for Graphemes {
    /// Creates a single `Graphemes` instance by concatenating multiple `Graphemes`.
    fn from_iter<I: IntoIterator<Item = Graphemes>>(iter: I) -> Self {
        let concatenated = iter.into_iter().flat_map(|g| g.0).collect();
        Graphemes(concatenated)
    }
}

impl FromIterator<Grapheme> for Graphemes {
    /// Creates a `Graphemes` instance from an iterator of `Grapheme`.
    fn from_iter<I: IntoIterator<Item = Grapheme>>(iter: I) -> Self {
        let mut g = Graphemes::default();
        for i in iter {
            g.push_back(i);
        }
        g
    }
}

impl Iterator for Graphemes {
    type Item = Grapheme;

    /// Pops a `Grapheme` from the front of the collection.
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<S: AsRef<str>> From<S> for Graphemes {
    /// Creates a `Graphemes` instance from a string slice, converting each char to a `Grapheme`.
    fn from(string: S) -> Self {
        string.as_ref().chars().map(Grapheme::new).collect()
    }
}

impl Len for Graphemes {
    /// Returns the number of `Grapheme` instances in the collection.
    fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the collection is empty.
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Graphemes {
    /// Calculates the total display width of all `Grapheme` instances in the collection.
    pub fn widths(&self) -> usize {
        self.0.iter().map(|grapheme| grapheme.width).sum()
    }
}
