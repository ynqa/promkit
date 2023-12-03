mod render;
pub use render::{History, Mode, Renderer, Suggest};
mod build;
pub use build::Builder;

#[derive(Clone)]
pub struct TextEditor {
    pub buf: String,
    pub position: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            // Set cursor
            buf: String::from(" "),
            position: 0,
        }
    }
}

impl TextEditor {
    pub fn content(&self) -> String {
        self.buf.clone()
    }

    pub fn content_without_cursor(&self) -> String {
        let mut ret = self.buf.clone();
        ret.pop();
        ret
    }

    pub fn masking(&self, mask: char) -> String {
        self.buf
            .chars()
            .enumerate()
            .map(|(i, c)| if i == self.buf.len() - 1 { c } else { mask })
            .collect::<String>()
    }

    fn is_head(&self) -> bool {
        self.position == 0
    }

    fn is_tail(&self) -> bool {
        self.position == self.buf.len() - 1
    }

    pub fn replace(&mut self, new: &str) {
        self.buf = new.to_owned();
        self.buf.push(' ');
        self.move_to_tail();
    }

    pub fn insert(&mut self, ch: char) {
        self.buf.insert(self.position, ch);
        self.forward();
    }

    pub fn overwrite(&mut self, ch: char) {
        if self.is_tail() {
            self.insert(ch)
        } else {
            self.buf
                .replace_range(self.position..self.position + 1, &ch.to_string());
            self.forward();
        }
    }

    pub fn erase(&mut self) {
        if !self.is_head() {
            self.backward();
            self.buf.drain(self.position..self.position + 1);
        }
    }

    pub fn erase_all(&mut self) {
        *self = Self::default();
    }

    pub fn move_to_head(&mut self) {
        self.position = 0;
    }

    pub fn move_to_tail(&mut self) {
        self.position = self.buf.len() - 1;
    }

    pub fn backward(&mut self) {
        if !self.is_head() {
            self.position -= 1;
        }
    }

    pub fn forward(&mut self) {
        if !self.is_tail() {
            self.position += 1;
        }
    }
}

#[cfg(test)]
mod test {
    mod masking {
        use super::super::*;

        #[test]
        fn test() {
            let txt = TextEditor {
                buf: String::from("abcde "),
                position: 0,
            };
            assert_eq!("***** ", txt.masking('*'))
        }
    }

    mod erase {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextEditor::default();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("bc "),
                position: 0, // indicate `b`.
            };
            txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let new = TextEditor {
                buf: String::from("ab "),
                position: 2, // indicate tail.
            };
            txt.erase();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_head() {
            let txt = TextEditor {
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
            let mut txt = TextEditor::default();
            let new = TextEditor {
                buf: String::from("d "),
                position: 1, // indicate tail.
            };
            txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("adbc "),
                position: 2, // indicate `b`.
            };
            txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let new = TextEditor {
                buf: String::from("abcd "),
                position: 4, // indicate tail.
            };
            txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let new = TextEditor {
                buf: String::from("dabc "),
                position: 1, // indicate `a`.
            };
            txt.insert('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }
    }

    mod overwrite {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            let new = TextEditor {
                buf: String::from("d "),
                position: 1, // indicate tail.
            };
            txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("adc "),
                position: 2, // indicate `c`.
            };
            txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let new = TextEditor {
                buf: String::from("abcd "),
                position: 4, // indicate tail.
            };
            txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let new = TextEditor {
                buf: String::from("dbc "),
                position: 1, // indicate `b`.
            };
            txt.overwrite('d');
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }
    }

    mod backward {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.backward();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            txt.backward();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 2, // indicate `c`.
            };
            txt.backward();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            txt.backward();
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod forward {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.forward();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 2, // indicate `c`.
            };
            txt.forward();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            txt.forward();
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            txt.forward();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }
    }

    mod to_head {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_head();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            txt.move_to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            txt.move_to_head();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            txt.move_to_head();
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(0, txt.position);
        }
    }

    mod to_tail {
        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_tail();
            assert_eq!(String::from(" "), txt.buf);
            assert_eq!(0, txt.position);
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 1, // indicate `b`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            txt.move_to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }

        #[test]
        fn test_at_tail() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            txt.move_to_tail();
            assert_eq!(String::from("abc "), txt.buf);
            assert_eq!(3, txt.position);
        }

        #[test]
        fn test_at_head() {
            let mut txt = TextEditor {
                buf: String::from("abc "),
                position: 0, // indicate `a`.
            };
            let new = TextEditor {
                buf: String::from("abc "),
                position: 3, // indicate tail.
            };
            txt.move_to_tail();
            assert_eq!(new.buf, txt.buf);
            assert_eq!(new.position, txt.position);
        }
    }
}
