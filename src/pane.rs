use crate::{grapheme::Graphemes, text::TextBuffer};

pub struct Requirement {
    /// Determina the order which the panes are drawn.
    pub priority_to_draw: u16,
    /// Determine the order which the panes are assigned vertical space
    /// when the available space is limited.
    pub priority_to_occupy_height: u16,
    /// Minimum amount of vertical space that the pane must occupy,
    /// even if the screen is not large enough to accommodate it fully.
    pub guaranteed_height: u16,
}

pub struct Pane {
    // pub requirement: Requirement,
    layout: Vec<Graphemes>,
    start_row_index: usize,
}

impl Pane {
    pub fn new(width: usize, textbuffer: &TextBuffer, label: &Graphemes) -> Self {
        let mut buf = vec![];
        buf.append(&mut label.clone());
        buf.append(&mut textbuffer.buf.clone());

        let mut layout = vec![];
        let mut row = Graphemes::default();
        for ch in buf.iter() {
            let width_with_next_char = row.iter().fold(0, |mut layout, g| {
                layout += g.width;
                layout
            }) + ch.width;
            if !row.is_empty() && width < width_with_next_char {
                layout.push(row);
                row = Graphemes::default();
            }
            if width >= ch.width {
                row.push(ch.clone());
            }
        }
        layout.push(row);
        Self {
            layout,
            start_row_index: textbuffer.position / width,
        }
    }

    pub fn extract(&self, viewport_height: usize) -> Vec<Graphemes> {
        if self.layout.len() <= viewport_height {
            return self.layout.clone();
        }
        let end_row_index = self.start_row_index + viewport_height;
        return self
            .layout
            .iter()
            .enumerate()
            .filter(|(i, _)| self.start_row_index <= *i && *i < end_row_index)
            .map(|(_i, row)| row.clone())
            .collect::<Vec<_>>();
    }
}

#[cfg(test)]
mod test {
    mod matrixify {
        use super::super::*;

        #[test]
        fn test() {
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" a"),
                Graphemes::from("aa"),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                Pane::new(
                    2,
                    &TextBuffer {
                        buf: Graphemes::from("aaa "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
                .layout
            );
        }

        #[test]
        fn test_with_emoji() {
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" "),
                Graphemes::from("😎"),
                Graphemes::from("😎"),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                Pane::new(
                    2,
                    &TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
                .layout
            );
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            let expect = vec![
                Graphemes::from(">"),
                Graphemes::from(">"),
                Graphemes::from(" "),
                Graphemes::from(" "),
            ];
            assert_eq!(
                expect,
                Pane::new(
                    1,
                    &TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
                .layout
            );
        }
    }
}
