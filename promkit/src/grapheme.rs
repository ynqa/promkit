use std::{
    collections::VecDeque,
    fmt,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use unicode_width::UnicodeWidthChar;

mod styled;
pub use styled::{matrixify, trim, StyledGraphemes};

impl From<char> for Grapheme {
    fn from(ch: char) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
        }
    }
}
/// Represents a single grapheme with its character and display width.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grapheme {
    ch: char,
    width: usize,
}

impl Grapheme {
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
        string.as_ref().chars().map(Grapheme::from).collect()
    }
}

impl fmt::Display for Graphemes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string: String = self.chars().iter().collect();
        write!(f, "{}", string)
    }
}

impl fmt::Debug for Graphemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for grapheme in self.iter() {
            write!(f, "{}", grapheme.ch)?;
        }
        Ok(())
    }
}

impl Graphemes {
    /// Calculates the total display width of all `Grapheme` instances in the collection.
    pub fn widths(&self) -> usize {
        self.0.iter().map(|grapheme| grapheme.width).sum()
    }

    /// Returns a `Vec<char>` containing the characters of all `Grapheme` instances in the collection.
    pub fn chars(&self) -> Vec<char> {
        self.0.iter().map(|grapheme| grapheme.ch).collect()
    }

    /// Replaces the specified range with the given string.
    pub fn replace_range<S: AsRef<str>>(&mut self, range: std::ops::Range<usize>, replacement: S) {
        // Remove the specified range.
        for _ in range.clone() {
            self.0.remove(range.start);
        }

        // Insert the replacement at the start of the range.
        let replacement_graphemes: Graphemes = replacement.as_ref().into();
        for grapheme in replacement_graphemes.0.iter().rev() {
            self.0.insert(range.start, grapheme.clone());
        }
    }
}
