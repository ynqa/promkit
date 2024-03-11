use crate::grapheme::StyledGraphemes;
pub struct Pane {
    /// The layout of graphemes within the pane.
    /// This vector stores the styled graphemes that make up the content of the pane.
    layout: Vec<StyledGraphemes>,
    /// The offset from the top of the pane, used when extracting graphemes to display.
    /// This value determines the starting point for grapheme extraction, allowing for scrolling behavior.
    offset: usize,
    /// An optional fixed height for the pane. If set, this limits the number of graphemes extracted.
    /// When specified, this restricts the maximum number of graphemes to be displayed, effectively setting the pane's height.
    fixed_height: Option<usize>,
}

impl Pane {
    /// Constructs a new `Pane` with the specified layout, offset, and optional fixed height.
    /// - `layout`: A vector of `StyledGraphemes` representing the content of the pane.
    /// - `offset`: The initial offset from the top of the pane.
    /// - `fixed_height`: An optional fixed height for the pane.
    pub fn new(layout: Vec<StyledGraphemes>, offset: usize, fixed_height: Option<usize>) -> Self {
        Pane {
            layout,
            offset,
            fixed_height,
        }
    }

    pub fn rows(&self) -> usize {
        self.layout.len()
    }

    /// Checks if the pane is empty.
    /// A pane is considered empty if it contains exactly one layout element with a width of 0.
    /// - Returns: `true` if the pane is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.layout.len() == 1 && self.layout[0].widths() == 0
    }

    /// Extracts a slice of the pane's layout to be displayed, based on the current offset and the viewport height.
    /// - `viewport_height`: The height of the viewport in which the pane is being displayed.
    /// - Returns: A vector of `StyledGraphemes` representing the visible portion of the pane.
    pub fn extract(&self, viewport_height: usize) -> Vec<StyledGraphemes> {
        let lines = self.layout.len().min(
            self.fixed_height
                .unwrap_or(viewport_height)
                .min(viewport_height),
        );

        let mut start = self.offset;
        let end = self.offset + lines;
        if end > self.layout.len() {
            start = self.layout.len().saturating_sub(lines);
        }

        return self
            .layout
            .iter()
            .enumerate()
            .filter(|(i, _)| start <= *i && *i < end)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
    }
}

#[cfg(test)]
mod test {
    mod is_empty {
        use crate::grapheme::matrixify;

        use super::super::*;

        #[test]
        fn test() {
            assert_eq!(
                true,
                Pane {
                    layout: matrixify(10, &StyledGraphemes::from("")),
                    offset: 0,
                    fixed_height: None,
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
                    fixed_height: None,
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
                    fixed_height: None,
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
                    fixed_height: None,
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
                    fixed_height: None,
                }
                .extract(3)
            );
        }

        #[test]
        fn test_with_small_fixed_height_and_beyond_extraction_size_and_offset_non_zero() {
            let expect = vec![
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
                    offset: 3, // indicate `dd`
                    fixed_height: Some(5),
                }
                .extract(4)
            );
        }

        #[test]
        fn test_with_large_fixed_height_and_beyond_extraction_size_and_offset_non_zero() {
            let expect = vec![
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
                    offset: 3, // indicate `dd`
                    fixed_height: Some(4),
                }
                .extract(5)
            );
        }
    }
}
