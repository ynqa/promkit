use crate::grapheme::StyledGraphemes;

#[derive(Clone)]
pub struct Pane {
    /// The layout of graphemes within the pane.
    /// This vector stores the styled graphemes that make up the content of the pane.
    layout: Vec<StyledGraphemes>,
    /// The offset from the top of the pane, used when extracting graphemes to display.
    /// This value determines the starting point for grapheme extraction, allowing for scrolling behavior.
    offset: usize,
}

impl Pane {
    /// Constructs a new `Pane` with the specified layout, offset, and optional fixed height.
    /// - `layout`: A vector of `StyledGraphemes` representing the content of the pane.
    /// - `offset`: The initial offset from the top of the pane.
    pub fn new(layout: Vec<StyledGraphemes>, offset: usize) -> Self {
        Pane { layout, offset }
    }

    pub fn visible_row_count(&self) -> usize {
        self.layout.len()
    }

    /// Checks if the pane is empty.
    pub fn is_empty(&self) -> bool {
        self.layout.is_empty()
    }

    pub fn extract(&self, viewport_height: usize) -> Vec<StyledGraphemes> {
        let lines = self.layout.len().min(viewport_height);
        let mut start = self.offset;
        let end = self.offset + lines;
        if end > self.layout.len() {
            start = self.layout.len().saturating_sub(lines);
        }

        self.layout
            .iter()
            .enumerate()
            .filter(|(i, _)| start <= *i && *i < end)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    mod visible_row_count {
        use crate::{crossterm::style::ContentStyle, text, PaneFactory};

        #[test]
        fn test() {
            let state = text::State {
                text: "".to_string(),
                style: ContentStyle::default(),
            };
            assert_eq!(0, state.create_pane(10, 10).visible_row_count())
        }
    }

    mod is_empty {
        use crate::grapheme::matrixify;

        use super::super::*;

        #[test]
        fn test() {
            assert_eq!(
                true,
                Pane {
                    layout: matrixify(10, 10, 0, &StyledGraphemes::from("")),
                    offset: 0,
                }
                .is_empty()
            );
        }
    }
    mod extract {
        use super::super::*;

        #[test]
        fn test_with_less_extraction_size_than_layout() {
            let expect = vec![
                StyledGraphemes::from("aa"),
                StyledGraphemes::from("bb"),
                StyledGraphemes::from("cc"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        StyledGraphemes::from("aa"),
                        StyledGraphemes::from("bb"),
                        StyledGraphemes::from("cc"),
                        StyledGraphemes::from("dd"),
                        StyledGraphemes::from("ee"),
                    ],
                    offset: 0,
                }
                .extract(3)
            );
        }

        #[test]
        fn test_with_much_extraction_size_than_layout() {
            let expect = vec![
                StyledGraphemes::from("aa"),
                StyledGraphemes::from("bb"),
                StyledGraphemes::from("cc"),
                StyledGraphemes::from("dd"),
                StyledGraphemes::from("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        StyledGraphemes::from("aa"),
                        StyledGraphemes::from("bb"),
                        StyledGraphemes::from("cc"),
                        StyledGraphemes::from("dd"),
                        StyledGraphemes::from("ee"),
                    ],
                    offset: 0,
                }
                .extract(10)
            );
        }

        #[test]
        fn test_with_within_extraction_size_and_offset_non_zero() {
            let expect = vec![StyledGraphemes::from("cc"), StyledGraphemes::from("dd")];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        StyledGraphemes::from("aa"),
                        StyledGraphemes::from("bb"),
                        StyledGraphemes::from("cc"),
                        StyledGraphemes::from("dd"),
                        StyledGraphemes::from("ee"),
                    ],
                    offset: 2, // indicate `cc`
                }
                .extract(2)
            );
        }

        #[test]
        fn test_with_beyond_extraction_size_and_offset_non_zero() {
            let expect = vec![
                StyledGraphemes::from("cc"),
                StyledGraphemes::from("dd"),
                StyledGraphemes::from("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        StyledGraphemes::from("aa"),
                        StyledGraphemes::from("bb"),
                        StyledGraphemes::from("cc"),
                        StyledGraphemes::from("dd"),
                        StyledGraphemes::from("ee"),
                    ],
                    offset: 3, // indicate `dd`
                }
                .extract(3)
            );
        }
    }
}
