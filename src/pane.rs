use crate::grapheme::Graphemes;

pub struct Pane {
    layout: Vec<Graphemes>,
    offset: usize,
}

impl Pane {
    pub fn new(layout: Vec<Graphemes>, offset: usize) -> Self {
        Pane { layout, offset }
    }

    pub fn extract(&self, viewport_height: usize) -> Vec<Graphemes> {
        if self.layout.len() <= viewport_height {
            return self.layout.clone();
        }
        let mut start = self.offset;
        let end = self.offset + viewport_height;
        if end > self.layout.len() {
            start = self.layout.len().saturating_sub(viewport_height);
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
        fn test() {
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
                }
                .extract(3)
            );
        }

        #[test]
        fn test_to_try_with_size_beyond() {
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
                }
                .extract(10)
            );
        }
    }
}
