//! Manages characters and their display widths within a terminal interface.
//!
//! This module provides structures
//! and functions for handling graphemes
//! (characters and their associated display widths)
//! in terminal applications.
//! It is designed to accurately manage cursor positions
//! and text rendering, especially when dealing
//! with wide characters such as emojis
//! or special symbols that occupy more than one column in terminal displays.
//!
//! # Structures
//!
//! - `Grapheme`: Represents a single character,
//! its display width, and optional styling.
//! - `Graphemes`: A collection of `Grapheme` instances,
//! supporting operations like total width calculation and styling.
//!
//! # Utility Functions
//!
//! - `matrixify`: Splits a collection of graphemes into lines
//! that fit within a specified width, useful for text wrapping.
//! - `trim`: Trims a collection of graphemes
//! to fit within a specified width, discarding any excess graphemes.
//!
//! # Usage
//!
//! This module is intended for use in terminal applications
//! where accurate text rendering and cursor movement are crucial.
//! It leverages the `unicode_width` crate
//! to calculate the display width of characters,
//! ensuring compatibility with a wide
//! range of Unicode characters.
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

/// Represents a single grapheme (character) with its display width and optional styling.
///
/// A grapheme may consist of a single character or a composed character sequence
/// that is treated as a single unit for display purposes. The display width is calculated
/// based on Unicode width standards, which helps in accurately positioning and rendering
/// text in terminal applications.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grapheme {
    ch: char,
    width: usize,
}

impl Grapheme {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

/// A collection of `Grapheme` instances.
///
/// Supports operations like calculating the total display width of the collection,
/// applying styles to individual graphemes, and generating a display representation
/// that respects the applied styles.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Graphemes(pub VecDeque<Grapheme>);

impl Deref for Graphemes {
    type Target = VecDeque<Grapheme>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Graphemes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Graphemes> for Graphemes {
    fn from_iter<I: IntoIterator<Item = Graphemes>>(iter: I) -> Self {
        let concatenated = iter.into_iter().flat_map(|g| g.0).collect();
        Graphemes(concatenated)
    }
}

impl FromIterator<Grapheme> for Graphemes {
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

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<S: AsRef<str>> From<S> for Graphemes {
    fn from(string: S) -> Self {
        string.as_ref().chars().map(Grapheme::new).collect()
    }
}

impl Len for Graphemes {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Graphemes {
    pub fn widths(&self) -> usize {
        self.0.iter().map(|grapheme| grapheme.width).sum()
    }
}
