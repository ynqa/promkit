use std::{
    collections::VecDeque,
    fmt,
    ops::{Deref, DerefMut},
};

use unicode_width::UnicodeWidthChar;

use crate::crossterm::style::{Attribute, ContentStyle};

use super::Graphemes;

/// Represents a single grapheme (character) with its display width and optional styling.
///
/// This structure is similar to `Grapheme` but includes styling information directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyledGrapheme {
    ch: char,
    width: usize,
    style: ContentStyle,
}

impl StyledGrapheme {
    pub fn new(ch: char, style: ContentStyle) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
            style,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn apply_style(&mut self, style: ContentStyle) {
        self.style = style;
    }
}

/// A collection of `StyledGrapheme` instances.
///
/// This structure supports operations like calculating the total display width of the collection
/// and generating a display representation that respects the applied styles.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct StyledGraphemes(pub VecDeque<StyledGrapheme>);

impl Deref for StyledGraphemes {
    type Target = VecDeque<StyledGrapheme>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyledGraphemes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<StyledGraphemes> for StyledGraphemes {
    fn from_iter<I: IntoIterator<Item = StyledGraphemes>>(iter: I) -> Self {
        let concatenated = iter
            .into_iter()
            .flat_map(|g| g.0.into_iter())
            .collect::<VecDeque<StyledGrapheme>>();
        StyledGraphemes(concatenated)
    }
}

impl FromIterator<StyledGrapheme> for StyledGraphemes {
    fn from_iter<I: IntoIterator<Item = StyledGrapheme>>(iter: I) -> Self {
        let mut g = StyledGraphemes::default();
        for i in iter {
            g.push_back(i);
        }
        g
    }
}

impl<S: AsRef<str>> From<S> for StyledGraphemes {
    fn from(string: S) -> Self {
        Self::from_str(string, ContentStyle::default())
    }
}

impl fmt::Debug for StyledGraphemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for styled_grapheme in self.iter() {
            write!(f, "{}", styled_grapheme.ch)?;
        }
        Ok(())
    }
}

impl StyledGraphemes {
    pub fn from_str<S: AsRef<str>>(string: S, style: ContentStyle) -> Self {
        string
            .as_ref()
            .chars()
            .map(|ch| StyledGrapheme::new(ch, style))
            .collect()
    }

    pub fn from_graphemes(graphemes: Graphemes, style: ContentStyle) -> Self {
        graphemes
            .map(|g| StyledGrapheme::new(g.ch, style))
            .collect()
    }

    pub fn widths(&self) -> usize {
        self.0
            .iter()
            .map(|styled_grapheme| styled_grapheme.width())
            .sum()
    }

    pub fn apply_style_at(mut self, idx: usize, style: ContentStyle) -> Self {
        if let Some(grapheme) = self.0.get_mut(idx) {
            grapheme.apply_style(style);
        }
        self
    }

    pub fn apply_attribute_to_all(mut self, attr: Attribute) -> Self {
        for styled_grapheme in &mut self.0 {
            styled_grapheme.style.attributes.set(attr);
        }
        self
    }

    pub fn styled_display(&self) -> StyledGraphemesDisplay<'_> {
        StyledGraphemesDisplay {
            styled_graphemes: self,
        }
    }
}

pub struct StyledGraphemesDisplay<'a> {
    styled_graphemes: &'a StyledGraphemes,
}

impl<'a> fmt::Display for StyledGraphemesDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for styled_grapheme in self.styled_graphemes.iter() {
            write!(f, "{}", styled_grapheme.style.apply(styled_grapheme.ch))?;
        }
        Ok(())
    }
}

/// Splits a collection of graphemes into lines that fit within a specified width.
///
/// This function is useful for text wrapping in terminal applications, ensuring that
/// lines do not exceed the specified width. It respects the display width of each grapheme,
/// allowing for accurate layout even with wide characters or emojis.
pub fn matrixify(width: usize, g: &StyledGraphemes) -> Vec<StyledGraphemes> {
    let mut ret = vec![];
    let mut row = StyledGraphemes::default();
    for styled in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + styled.width;
        if !row.is_empty() && width < width_with_next_char {
            ret.push(row);
            row = StyledGraphemes::default();
        }
        if width >= styled.width {
            row.push_back(styled.clone());
        }
    }
    ret.push(row);
    ret
}

/// Trims a collection of graphemes to fit within a specified width.
///
/// This function discards any excess graphemes that would cause the total display width
/// to exceed the specified limit. It is useful for ensuring that a piece of text fits
/// within a given space without wrapping.
pub fn trim(width: usize, g: &StyledGraphemes) -> StyledGraphemes {
    let mut row = StyledGraphemes::default();
    for ch in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + ch.width;
        if width < width_with_next_char {
            break;
        }
        if width >= ch.width {
            row.push_back(ch.clone());
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
                StyledGraphemes::from(">>"),
                StyledGraphemes::from(" a"),
                StyledGraphemes::from("aa"),
                StyledGraphemes::from(" "),
            ];
            assert_eq!(expect, matrixify(2, &StyledGraphemes::from(">> aaa ")),);
        }

        #[test]
        fn test_with_emoji() {
            let expect = vec![
                StyledGraphemes::from(">>"),
                StyledGraphemes::from(" "),
                StyledGraphemes::from("😎"),
                StyledGraphemes::from("😎"),
                StyledGraphemes::from(" "),
            ];
            assert_eq!(expect, matrixify(2, &StyledGraphemes::from(">> 😎😎 ")),);
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            let expect = vec![
                StyledGraphemes::from(">"),
                StyledGraphemes::from(">"),
                StyledGraphemes::from(" "),
                StyledGraphemes::from(" "),
            ];
            assert_eq!(expect, matrixify(1, &StyledGraphemes::from(">> 😎😎 ")),);
        }
    }

    mod trim {
        use super::super::*;

        #[test]
        fn test() {
            assert_eq!(
                StyledGraphemes::from(">> a"),
                trim(4, &StyledGraphemes::from(">> aaa "))
            );
        }

        #[test]
        fn test_with_emoji() {
            assert_eq!(
                StyledGraphemes::from("😎"),
                trim(2, &StyledGraphemes::from("😎"))
            );
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            assert_eq!(
                StyledGraphemes::from(""),
                trim(1, &StyledGraphemes::from("😎"))
            );
        }
    }
}
