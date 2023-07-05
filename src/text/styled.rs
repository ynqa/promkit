use crate::{crossterm::style::Color, grapheme::Graphemes};

use super::TextBuffer;

pub struct StyledTextBuffer<'t> {
    text_buffer: &'t TextBuffer,
    label: Graphemes,
    label_color: Color,
    cursor_color: Color,
}

impl<'t> StyledTextBuffer<'t> {
    pub fn new(
        text_buffer: &'t TextBuffer,
        label: Graphemes,
        label_color: Color,
        cursor_color: Color,
    ) -> Self {
        Self {
            text_buffer,
            label,
            label_color,
            cursor_color,
        }
    }

    pub fn matrixify(&self, width: u16) -> Vec<Graphemes> {
        let mut buf = vec![];
        buf.append(&mut self.label.clone());
        buf.append(&mut self.text_buffer.buf.clone());

        let mut res = vec![];
        let mut row = Graphemes::default();
        for ch in buf.iter() {
            let width_with_next_char = row.iter().fold(0, |mut res, g| {
                res += g.width;
                res
            }) + ch.width;
            if width < width_with_next_char as u16 {
                res.push(row);
                row = Graphemes::default();
            }
            row.push(ch.clone());
        }
        res.push(row);
        res
    }
}

#[cfg(test)]
mod test {
    mod matrixify {
        use super::super::*;

        #[test]
        fn test() {
            let txt = StyledTextBuffer {
                text_buffer: &TextBuffer {
                    buf: Graphemes::from("aaa "),
                    position: 0,
                },
                label: Graphemes::from(">> "),
                label_color: Color::Reset,
                cursor_color: Color::Reset,
            };
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" a"),
                Graphemes::from("aa"),
                Graphemes::from(" "),
            ];
            assert_eq!(expect, txt.matrixify(2));
        }

        #[test]
        fn test_with_emoji() {
            let txt = StyledTextBuffer {
                text_buffer: &TextBuffer {
                    buf: Graphemes::from("😎😎 "),
                    position: 0,
                },
                label: Graphemes::from(">> "),
                label_color: Color::Reset,
                cursor_color: Color::Reset,
            };
            let expect = vec![
                Graphemes::from(">>"),
                Graphemes::from(" "),
                Graphemes::from("😎"),
                Graphemes::from("😎"),
                Graphemes::from(" "),
            ];
            assert_eq!(expect, txt.matrixify(2));
        }
    }
}
