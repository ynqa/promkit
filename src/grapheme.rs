//! # Grapheme
//!
//! `grapheme` manages the characters and their width at the display.
//!
//! Note that to manage the width of character is
//! in order to consider how many the positions of cursor should be moved
//! when e.g. emojis and the special characters are displayed on the terminal.
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

use radix_trie::TrieKey;
use unicode_width::UnicodeWidthChar;

/// A character and its width.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Grapheme {
    pub ch: char,
    pub width: usize,
}

impl From<char> for Grapheme {
    fn from(c: char) -> Self {
        Self {
            ch: c,
            width: UnicodeWidthChar::width(c).unwrap_or(0),
        }
    }
}

/// Characters and their width.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Graphemes(pub Vec<Grapheme>);

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

impl<S: Into<String>> From<S> for Graphemes {
    fn from(s: S) -> Self {
        s.into().chars().map(Grapheme::from).collect()
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

impl Display for Graphemes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .fold(String::new(), |s, g| format!("{}{}", s, g.ch))
        )
    }
}

impl Graphemes {
    pub fn width(&self) -> usize {
        self.iter().fold(0, |mut c, g| {
            c += g.width;
            c
        })
    }

    pub fn longest_common_prefix(&self, g: &Graphemes) -> Graphemes {
        self.iter()
            .zip(g.iter())
            .take_while(|&(a, b)| a == b)
            .map(|(a, _)| a.clone())
            .collect()
    }
}

#[test]
fn longest_common_prefix() {
    assert_eq!(
        Graphemes::from("ab"),
        Graphemes::from("ab").longest_common_prefix(&Graphemes::from("abc")),
    );

    assert_eq!(
        Graphemes::from("ab"),
        Graphemes::from("abc").longest_common_prefix(&Graphemes::from("ab")),
    );

    assert_eq!(
        Graphemes::default(),
        Graphemes::from("abc").longest_common_prefix(&Graphemes::default()),
    );

    assert_eq!(
        Graphemes::default(),
        Graphemes::default().longest_common_prefix(&Graphemes::from("abc")),
    );

    assert_eq!(
        Graphemes::default(),
        Graphemes::default().longest_common_prefix(&Graphemes::default()),
    );
}
