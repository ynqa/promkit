use crate::grapheme::Graphemes;

pub struct Pane {
    pub layout: Vec<Graphemes>,
    pub offset: usize,
}

impl Pane {
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
                Graphemes::from("aa"),
                Graphemes::from("bb"),
                Graphemes::from("cc"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::from("aa"),
                        Graphemes::from("bb"),
                        Graphemes::from("cc"),
                        Graphemes::from("dd"),
                        Graphemes::from("ee"),
                    ],
                    offset: 0,
                }
                .extract(3)
            );
        }

        #[test]
        fn test2() {
            let expect = vec![
                Graphemes::from("aa"),
                Graphemes::from("bb"),
                Graphemes::from("cc"),
                Graphemes::from("dd"),
                Graphemes::from("ee"),
            ];
            assert_eq!(
                expect,
                Pane {
                    layout: vec![
                        Graphemes::from("aa"),
                        Graphemes::from("bb"),
                        Graphemes::from("cc"),
                        Graphemes::from("dd"),
                        Graphemes::from("ee"),
                    ],
                    offset: 0,
                }
                .extract(10)
            );
        }
    }
}
