use std::fmt;

use crate::{crossterm::style::ContentStyle, grapheme::Graphemes};

#[derive(Clone, Debug, PartialEq)]
pub struct TextBuffer {
    buf: String,
    pub position: usize,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            // Set cursor
            buf: String::from(" "),
            position: 0,
        }
    }
}

impl fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = self.buf.clone();
        ret.pop();
        write!(f, "{}", ret)
    }
}

impl TextBuffer {
    pub fn graphemes(&self, base: ContentStyle, cursor: ContentStyle) -> Graphemes {
        Graphemes::new_with_style(&self.buf, base).stylize(self.position, cursor)
    }

    fn is_head(&self) -> bool {
        self.position == 0
    }

    fn is_tail(&self) -> bool {
        self.position == self.buf.len() - 1
    }

    pub fn replace(&mut self, new: &str) -> [Self; 2] {
        let prev = self.clone();
        self.buf = new.to_owned();
        self.buf.push(' ');
        self.move_to_tail();
        [prev, self.clone()]
    }

    pub fn insert(&mut self, ch: char) -> [Self; 2] {
        let prev = self.clone();
        self.buf.insert(self.position, ch);
        self.next();
        [prev, self.clone()]
    }

    pub fn overwrite(&mut self, ch: char) -> [Self; 2] {
        let prev = self.clone();
        if self.is_tail() {
            self.insert(ch)
        } else {
            self.buf
                .replace_range(self.position..self.position + 1, &ch.to_string());
            self.next();
            [prev, self.clone()]
        }
    }

    pub fn erase(&mut self) -> [Self; 2] {
        let prev = self.clone();
        if !self.is_head() {
            self.prev();
            self.buf.drain(self.position..self.position + 1);
        }
        [prev, self.clone()]
    }

    pub fn erase_all(&mut self) -> [Self; 2] {
        let prev = self.clone();
        *self = Self::default();
        [prev, self.clone()]
    }

    pub fn move_to_head(&mut self) -> [Self; 2] {
        let prev = self.clone();
        self.position = 0;
        [prev, self.clone()]
    }

    pub fn move_to_tail(&mut self) -> [Self; 2] {
        let prev = self.clone();
        self.position = self.buf.len() - 1;
        [prev, self.clone()]
    }

    pub fn prev(&mut self) -> [Self; 2] {
        let prev = self.clone();
        if !self.is_head() {
            self.position -= 1;
        }
        [prev, self.clone()]
    }

    pub fn next(&mut self) -> [Self; 2] {
        let prev = self.clone();
        if !self.is_tail() {
            self.position += 1;
        }
        [prev, self.clone()]
    }
}

#[cfg(test)]
mod test {
    mod erase {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextBuffer::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("bc "),
                position: 0, // indicate `b`.
            };
            let diff = txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("ab "),
                position: 2, // indicate tail.
            };
            let diff = txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_head() {
            let txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod insert {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::default();
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("d "),
                position: 1, // indicate tail.
            };
            let diff = txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("adbc "),
                position: 2, // indicate `b`.
            };
            let diff = txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abcd "),
                position: 4, // indicate tail.
            };
            let diff = txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("dabc "),
                position: 1, // indicate `a`.
            };
            let diff = txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }
    }

    mod overwrite {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextBuffer::default();
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("d "),
                position: 1, // indicate tail.
            };
            let diff = txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("adc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abcd "),
                position: 4, // indicate tail.
            };
            let diff = txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("dbc "),
                position: 1, // indicate `b`.
            };
            let diff = txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }
    }

    mod prev {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextBuffer::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.prev();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.prev();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_head() {
            let txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod next {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextBuffer::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 2, // indicate `c`.
            };
            let diff = txt.next();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let diff = txt.next();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }
    }

    mod to_head {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextBuffer::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.move_to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let diff = txt.move_to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_head() {
            let txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod to_tail {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextBuffer::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let diff = txt.move_to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }

        #[test]
        fn test_at_tail() {
            let txt = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextBuffer {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let old = txt.clone();
            let new = TextBuffer {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let diff = txt.move_to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
            assert_eq!(diff, [old, new]);
        }
    }
}
