use crate::grapheme::Graphemes;

pub struct Pane {
    layout: Vec<Graphemes>,
    offset: usize,
    fixed_height: Option<usize>,
}

impl Pane {
    pub fn new(layout: Vec<Graphemes>, offset: usize, fixed_height: Option<usize>) -> Self {
        Pane {
            layout,
            offset,
            fixed_height,
        }
    }

    pub fn extract(&self, remaining_height: usize) -> Vec<Graphemes> {
        let lines = self.layout.len().min(
            self.fixed_height
                .unwrap_or(remaining_height)
                .min(remaining_height),
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
    mod extract {
        use super::super::*;

        #[test]
        fn test_with_less_extraction_size_than_layout() {
            let expect = vec![
                Graphemes::new("aa"),
                Graphemes::new("bb"),
                Graphemes::new("cc"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
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
                Graphemes::new("aa"),
                Graphemes::new("bb"),
                Graphemes::new("cc"),
                Graphemes::new("dd"),
                Graphemes::new("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
                    ],
                    offset: 0,
                    fixed_height: None,
                }
                .extract(10)
            );
        }

        #[test]
        fn test_with_within_extraction_size_and_offset_non_zero() {
            let expect = vec![Graphemes::new("cc"), Graphemes::new("dd")];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
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
                Graphemes::new("cc"),
                Graphemes::new("dd"),
                Graphemes::new("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
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
                Graphemes::new("bb"),
                Graphemes::new("cc"),
                Graphemes::new("dd"),
                Graphemes::new("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
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
                Graphemes::new("bb"),
                Graphemes::new("cc"),
                Graphemes::new("dd"),
                Graphemes::new("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::new("aa"),
                        Graphemes::new("bb"),
                        Graphemes::new("cc"),
                        Graphemes::new("dd"),
                        Graphemes::new("ee"),
                    ],
                    offset: 3, // indicate `dd`
                    fixed_height: Some(4),
                }
                .extract(5)
            );
        }
    }
}
