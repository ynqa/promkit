use anyhow::Ok;

use crate::{grapheme::Graphemes, text::TextBuffer, Result};

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
}

impl Pane {
    fn matrixify(&self, width: u16, textbuffer: &TextBuffer, label: &Graphemes) -> Vec<Graphemes> {
        let mut buf = vec![];
        buf.append(&mut label.clone());
        buf.append(&mut textbuffer.buf.clone());

        let mut res = vec![];
        let mut row = Graphemes::default();
        for ch in buf.iter() {
            let width_with_next_char = row.iter().fold(0, |mut res, g| {
                res += g.width;
                res
            }) + ch.width;
            if !row.is_empty() && width < width_with_next_char as u16 {
                res.push(row);
                row = Graphemes::default();
            }
            if width >= ch.width as u16 {
                row.push(ch.clone());
            }
        }
        res.push(row);
        res
    }

    pub fn render(
        &mut self,
        viewport: (u16, u16),
        textbuffer: &TextBuffer,
        label: &Graphemes,
    ) -> Result<Vec<Graphemes>> {
        Ok(self.matrixify(viewport.0, textbuffer, label))
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
                Pane {}.matrixify(
                    2,
                    &TextBuffer {
                        buf: Graphemes::from("aaa "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
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
                Pane {}.matrixify(
                    2,
                    &TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
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
                Pane {}.matrixify(
                    1,
                    &TextBuffer {
                        buf: Graphemes::from("😎😎 "),
                        position: 0,
                    },
                    &Graphemes::from(">> "),
                )
            );
        }
    }
}
