use std::fmt;

use crate::grapheme::{Grapheme, Graphemes};

#[derive(Clone, Debug, PartialEq)]
pub struct TextBuffer {
    buf: Graphemes,
    position: u16,
}

impl fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.buf)
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            // Set cursor
            buf: Graphemes::from(" "),
            position: 0,
        }
    }

    pub fn text(&self) -> &Graphemes {
        &self.buf
    }

    fn is_head(&self) -> bool {
        self.position == 0
    }

    fn is_tail(&self) -> bool {
        self.position == self.buf.len() as u16 - 1
    }

    pub fn resize(&mut self, _size: (u16, u16)) {
        self.position = 0
    }

    fn replace(&mut self, new: &Graphemes) {
        self.buf = new.clone();
        self.buf.push(Grapheme::from(' '));
        self.to_tail();
    }

    pub fn insert(&mut self, ch: Grapheme) -> Option<[Self; 2]> {
        let prev = self.clone();
        self.buf.insert(self.position as usize, ch);
        self.next();
        Some([prev, self.clone()])
    }

    pub fn overwrite(&mut self, ch: Grapheme) -> Option<[Self; 2]> {
        let prev = self.clone();
        if self.is_tail() {
            self.insert(ch)
        } else {
            self.buf.splice(
                self.position as usize..(self.position + 1) as usize,
                vec![ch],
            );
            self.next();
            Some([prev, self.clone()])
        }
    }

    pub fn erase(&mut self) -> Option<[Self; 2]> {
        let prev = self.clone();
        if !self.is_head() {
            self.prev();
            self.buf
                .drain(self.position as usize..(self.position + 1) as usize);
            return Some([prev, self.clone()]);
        }
        None
    }

    pub fn to_head(&mut self) -> Option<[Self; 2]> {
        let prev = self.clone();
        if !self.is_head() {
            self.position = 0;
            return Some([prev, self.clone()]);
        }
        None
    }

    pub fn to_tail(&mut self) -> Option<[Self; 2]> {
        let prev = self.clone();
        if !self.is_tail() {
            self.position = self.buf.len() as u16 - 1;
            return Some([prev, self.clone()]);
        }
        None
    }

    pub fn prev(&mut self) -> Option<[Self; 2]> {
        let prev = self.clone();
        if !self.is_head() {
            self.position -= 1;
            return Some([prev, self.clone()]);
        }
        None
    }

    pub fn next(&mut self) -> Option<[Self; 2]> {
        let prev = self.clone();
        if !self.is_tail() {
            self.position += 1;
            return Some([prev, self.clone()]);
        }
        None
    }

    pub fn matrixify(&self, width: u16) -> Vec<Graphemes> {
        let mut res = vec![];
        let mut row = Graphemes::default();
        for ch in self.buf.iter() {
            let current_row_width = row.iter().fold(0, |mut res, g| {
                res += g.width;
                res
            });
            if width <= current_row_width as u16 {
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
    mod erase {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            assert!(txt.erase().is_none());
            assert_eq!(Graphemes::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("bc "),
                position: 0, // indicate `b`.
            };
            let diff = txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("ab "),
                position: 2, // indicate tail.
            };
            let diff = txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            assert!(txt.erase().is_none());
            assert_eq!(Graphemes::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod insert {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("d "),
                position: 1, // indicate tail.
            };
            let diff = txt.insert(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("adbc "),
                position: 2, // indicate `b`.
            };
            let diff = txt.insert(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abcd "),
                position: 4, // indicate tail.
            };
            let diff = txt.insert(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("dabc "),
                position: 1, // indicate `a`.
            };
            let diff = txt.insert(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }
    }

    mod overwrite {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("d "),
                position: 1, // indicate tail.
            };
            let diff = txt.overwrite(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("adc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.overwrite(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abcd "),
                position: 4, // indicate tail.
            };
            let diff = txt.overwrite(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("dbc "),
                position: 1, // indicate `b`.
            };
            let diff = txt.overwrite(Grapheme::from('d'));
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }
    }

    mod prev {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            assert!(txt.prev().is_none());
            assert_eq!(Graphemes::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.prev();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.prev();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            assert!(txt.prev().is_none());
            assert_eq!(Graphemes::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod next {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            assert!(txt.next().is_none());
            assert_eq!(Graphemes::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.next();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            assert!(txt.next().is_none());
            assert_eq!(Graphemes::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let diff = txt.next();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }
    }

    mod to_head {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            assert!(txt.to_head().is_none());
            assert_eq!(Graphemes::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            assert!(txt.to_head().is_none());
            assert_eq!(Graphemes::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod to_tail {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::new();
            assert!(txt.to_tail().is_none());
            assert_eq!(Graphemes::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let diff = txt.to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            assert!(txt.to_tail().is_none());
            assert_eq!(Graphemes::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: Graphemes::from("abc "),
                position: 3, // indicate tail.
            };
            let diff = txt.to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff.unwrap(), [old, new]);
        }
    }

    mod matrixify {
        use super::super::*;

        #[test]
        fn test() {
            let txt = TextBuffer {
                buf: Graphemes::from("aaa "),
                position: 0,
            };
            let expect = vec![Graphemes::from("aa"), Graphemes::from("a ")];
            assert_eq!(expect, txt.matrixify(2));
        }

        #[test]
        fn test_with_emoji() {
            let txt = TextBuffer {
                buf: Graphemes::from("😎😎 "),
                position: 0,
            };
            let expect = vec![
                Graphemes::from("😎"),
                Graphemes::from("😎"),
                Graphemes::from(" "),
            ];
            assert_eq!(expect, txt.matrixify(2));
        }
    }
}
