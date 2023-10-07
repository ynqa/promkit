//! # Grapheme
//!
//! `grapheme` manages the characters and their width at the display.
//!
//! Note that to manage the width of character is
//! in order to consider how many the positions of cursor should be moved
//! when e.g. emojis and the special characters are displayed on the terminal.
use std::{
    fmt,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use radix_trie::TrieKey;
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

    pub fn new_with_style(ch: char, style: ContentStyle) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
            style,
        }
    }
}

/// Characters and their width.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    pub fn display<'a>(&'a self) -> StyledGraphemesDisplay<'a> {
        StyledGraphemesDisplay { graphemes: self }
    }

    pub fn to_string(&self) -> String {
        self.iter().fold(String::new(), |agg, grapheme| {
            format!("{}{}", agg, grapheme.ch)
        })
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

impl TrieKey for Graphemes {
    fn encode_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
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

pub fn matrixify(width: usize, g: Graphemes) -> Vec<Graphemes> {
    let mut ret = vec![];
    let mut row = Graphemes::default();
    for ch in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + ch.width;
        if !row.is_empty() && (width as usize) < width_with_next_char {
            ret.push(row);
            row = Graphemes::default();
        }
        if (width as usize) >= ch.width {
            row.push(ch.clone());
        }
    }
    ret.push(row);
    ret
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
            assert_eq!(expect, matrixify(2, Graphemes::new(">> aaa ")),);
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
            assert_eq!(expect, matrixify(2, Graphemes::new(">> 😎😎 ")),);
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            let expect = vec![
                Graphemes::new(">"),
                Graphemes::new(">"),
                Graphemes::new(" "),
                Graphemes::new(" "),
            ];
            assert_eq!(expect, matrixify(1, Graphemes::new(">> 😎😎 ")),);
        }
    }
}
