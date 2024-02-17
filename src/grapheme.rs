//! Manages characters and their display widths within a terminal interface.
//!
//! This module provides structures and functions for handling graphemes (characters and their associated display widths)
//! in terminal applications. It is designed to accurately manage cursor positions and text rendering, especially when dealing
//! with wide characters such as emojis or special symbols that occupy more than one column in terminal displays.
//!
//! # Structures
//!
//! - `Grapheme`: Represents a single character, its display width, and optional styling.
//! - `Graphemes`: A collection of `Grapheme` instances, supporting operations like total width calculation and styling.
//!
//! # Utility Functions
//!
//! - `matrixify`: Splits a collection of graphemes into lines that fit within a specified width, useful for text wrapping.
//! - `trim`: Trims a collection of graphemes to fit within a specified width, discarding any excess graphemes.
//!
//! # Usage
//!
//! This module is intended for use in terminal applications where accurate text rendering and cursor movement are crucial.
//! It leverages the `unicode_width` crate to calculate the display width of characters, ensuring compatibility with a wide
//! range of Unicode characters.
use std::{
    fmt::{self, Debug},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use unicode_width::UnicodeWidthChar;

use crate::crossterm::style::ContentStyle;

/// A character and its width.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grapheme {
    ch: char,
    width: usize,
    style: ContentStyle,
}

impl Grapheme {
    pub fn new(ch: char) -> Self {
        Grapheme::new_with_style(ch, ContentStyle::new())
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn new_with_style(ch: char, style: ContentStyle) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
            style,
        }
    }
}

/// Characters and their width.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Graphemes(pub Vec<Grapheme>);

impl Graphemes {
    pub fn new<S: AsRef<str>>(string: S) -> Self {
        Graphemes::new_with_style(string, ContentStyle::new())
    }

    pub fn new_with_style<S: AsRef<str>>(string: S, style: ContentStyle) -> Self {
        string
            .as_ref()
            .chars()
            .map(|ch| Grapheme::new_with_style(ch, style))
            .collect()
    }

    pub fn widths(&self) -> usize {
        self.0.iter().map(|grapheme| grapheme.width).sum()
    }

    pub fn stylize(mut self, idx: usize, style: ContentStyle) -> Self {
        self.get_mut(idx).map(|grapheme| {
            grapheme.style = style;
            grapheme
        });
        self
    }

    pub fn styled_display(&self) -> StyledGraphemesDisplay<'_> {
        StyledGraphemesDisplay { graphemes: self }
    }
}

impl Debug for Graphemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for grapheme in self.iter() {
            write!(f, "{}", grapheme.ch)?;
        }
        Ok(())
    }
}

impl Deref for Graphemes {
    type Target = Vec<Grapheme>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Graphemes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Grapheme> for Graphemes {
    fn from_iter<I: IntoIterator<Item = Grapheme>>(iter: I) -> Self {
        let mut g = Graphemes::default();
        for i in iter {
            g.push(i);
        }
        g
    }
}

pub struct StyledGraphemesDisplay<'a> {
    graphemes: &'a Graphemes,
}

impl<'a> fmt::Display for StyledGraphemesDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for grapheme in self.graphemes.iter() {
            write!(f, "{}", grapheme.style.apply(grapheme.ch))?;
        }
        Ok(())
    }
}

pub fn matrixify(width: usize, g: &Graphemes) -> Vec<Graphemes> {
    let mut ret = vec![];
    let mut row = Graphemes::default();
    for ch in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + ch.width;
        if !row.is_empty() && width < width_with_next_char {
            ret.push(row);
            row = Graphemes::default();
        }
        if width >= ch.width {
            row.push(ch.clone());
        }
    }
    ret.push(row);
    ret
}

pub fn trim(width: usize, g: &Graphemes) -> Graphemes {
    let mut row = Graphemes::default();
    for ch in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + ch.width;
        if width < width_with_next_char {
            break;
        }
        if width >= ch.width {
            row.push(ch.clone());
        }
    }
    row
}

#[cfg(test)]
mod test {
    mod matrixify {
        use super::super::*;

        #[test]
        fn test() {
            let expect = vec![
                Graphemes::new(">>"),
                Graphemes::new(" a"),
                Graphemes::new("aa"),
                Graphemes::new(" "),
            ];
            assert_eq!(expect, matrixify(2, &Graphemes::new(">> aaa ")),);
        }

        #[test]
        fn test_with_emoji() {
            let expect = vec![
                Graphemes::new(">>"),
                Graphemes::new(" "),
                Graphemes::new("😎"),
                Graphemes::new("😎"),
                Graphemes::new(" "),
            ];
            assert_eq!(expect, matrixify(2, &Graphemes::new(">> 😎😎 ")),);
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            let expect = vec![
                Graphemes::new(">"),
                Graphemes::new(">"),
                Graphemes::new(" "),
                Graphemes::new(" "),
            ];
            assert_eq!(expect, matrixify(1, &Graphemes::new(">> 😎😎 ")),);
        }
    }

    mod trim {
        use super::super::*;

        #[test]
        fn test() {
            assert_eq!(Graphemes::new(">> a"), trim(4, &Graphemes::new(">> aaa ")));
        }

        #[test]
        fn test_with_emoji() {
            assert_eq!(Graphemes::new("😎"), trim(2, &Graphemes::new("😎")));
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            assert_eq!(Graphemes::new(""), trim(1, &Graphemes::new("😎")));
        }
    }
}
