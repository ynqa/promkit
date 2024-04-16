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

impl ToString for StyledGrapheme {
    fn to_string(&self) -> String {
        self.ch.to_string()
    }
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

impl ToString for StyledGraphemes {
    fn to_string(&self) -> String {
        self.iter().map(|g| g.ch).collect()
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

pub fn matrixify(
    width: usize,
    height: usize,
    offset: usize,
    g: &StyledGraphemes,
) -> Vec<StyledGraphemes> {
    let mut all = vec![];
    let mut row = StyledGraphemes::default();
    for styled in g.iter() {
        let width_with_next_char = row.iter().fold(0, |mut layout, g| {
            layout += g.width;
            layout
        }) + styled.width;
        if !row.is_empty() && width < width_with_next_char {
            all.push(row);
            row = StyledGraphemes::default();
        }
        if width >= styled.width {
            row.push_back(styled.clone());
        }
    }
    if !row.is_empty() {
        all.push(row);
    }

    // Adjusting the start and end indices for slicing the `all` vector.
    // The goal is to filter the vector to include elements from `offset` to `offset + height`.
    // However, if `offset + height` exceeds the length of `all`, we adjust to ensure the slice
    // does not go out of bounds. The `end` is set to the minimum of `offset + height` and `all.len()`,
    // ensuring we do not exceed the vector's length. The `start` is calculated to ensure we capture
    // a slice of length up to `height`, but adjusted to not underflow if `end` is less than `height`.
    // If the calculated range exceeds the vector's bounds, `start` defaults to 0, effectively
    // adjusting the range to fit within `0..all.len()`, thus ensuring we always return a valid slice
    // of the vector, either fitting the desired range or adjusted to the vector's size if the range is too large.
    let end = std::cmp::min(offset + height, all.len());
    let start = if end > height { end - height } else { 0 };

    all.iter()
        .enumerate()
        .filter(|(i, _)| start <= *i && *i < end)
        .map(|(_, row)| row.clone())
        .collect::<Vec<_>>()
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
    use super::*;

    mod matrixify {
        use super::*;

        #[test]
        fn test_with_single_line() {
            let input = StyledGraphemes::from("Hello, world!");
            let result = matrixify(12, 1, 0, &input);
            assert_eq!(1, result.len());
            assert_eq!("Hello, world", result[0].to_string());
        }

        #[test]
        fn test_with_multiple_lines() {
            let input = StyledGraphemes::from("Hello, world! This is a test.");
            let result = matrixify(10, 3, 0, &input);
            assert_eq!(3, result.len());
            assert_eq!("Hello, wor", result[0].to_string());
            assert_eq!("ld! This i", result[1].to_string());
            assert_eq!("s a test.", result[2].to_string());
        }

        #[test]
        fn test_with_offset() {
            let input = StyledGraphemes::from("One Two Three Four Five");
            let result = matrixify(8, 2, 1, &input);
            assert_eq!(2, result.len());
            assert_eq!("Three Fo", result[0].to_string());
            assert_eq!("ur Five", result[1].to_string());
        }

        #[test]
        fn test_with_offset_and_compensation() {
            let input = StyledGraphemes::from("One Two Three Four Five");
            let result = matrixify(8, 100, 1, &input);
            assert_eq!(3, result.len());
            assert_eq!("One Two ", result[0].to_string());
            assert_eq!("Three Fo", result[1].to_string());
            assert_eq!("ur Five", result[2].to_string());
        }

        #[test]
        fn test_with_empty_input() {
            let input = StyledGraphemes::default();
            let result = matrixify(10, 2, 0, &input);
            assert!(result.is_empty());
        }

        #[test]
        fn test_with_width_smaller_than_any_grapheme() {
            let input = StyledGraphemes::from("12345");
            let result = matrixify(1, 5, 0, &input);
            assert_eq!(5, result.len());
            for (i, line) in result.iter().enumerate() {
                assert_eq!(input[i].to_string(), line.to_string());
            }
        }

        #[test]
        fn test_with_height_less_than_needed() {
            let input = StyledGraphemes::from("Hello, world! This is a test.");
            let result = matrixify(10, 1, 0, &input);
            assert_eq!(1, result.len());
            assert_eq!("Hello, wor", result[0].to_string());
        }

        #[test]
        fn test_with_large_offset() {
            let input = StyledGraphemes::from("Hello, world! This is a test.");
            let result = matrixify(10, 2, 5, &input);
            assert_eq!(2, result.len());
            assert_eq!("ld! This i", result[0].to_string());
            assert_eq!("s a test.", result[1].to_string());
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
