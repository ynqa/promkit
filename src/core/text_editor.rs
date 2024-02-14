mod history;
pub use history::History;
mod render;
pub use render::Renderer;
mod suggest;
pub use suggest::Suggest;
mod mode;
pub use mode::Mode;
mod build;
pub use build::Builder;

use crate::core::cursor::Cursor;

#[derive(Clone)]
pub struct TextEditor(Cursor<String>);

impl Default for TextEditor {
    fn default() -> Self {
        Self(Cursor::new(
            // Set cursor
            String::from(" "),
        ))
    }
}

impl TextEditor {
    pub fn text(&self) -> String {
        self.0.contents().clone()
    }

    pub fn text_without_cursor(&self) -> String {
        let mut ret = self.text();
        ret.pop();
        ret
    }

    pub fn position(&self) -> usize {
        self.0.position()
    }
    pub fn masking(&self, mask: char) -> String {
        self.text()
            .chars()
            .enumerate()
            .map(|(i, c)| if i == self.text().len() - 1 { c } else { mask })
            .collect::<String>()
    }

    pub fn replace(&mut self, new: &str) {
        let mut buf = new.to_owned();
        buf.push(' ');
        let pos = buf.len() - 1;
        *self = Self(Cursor::new_with_position(buf, pos));
    }

    pub fn insert(&mut self, ch: char) {
        let pos = self.position();
        self.0.contents_mut().insert(pos, ch);
        self.forward();
    }

    pub fn overwrite(&mut self, ch: char) {
        if self.0.is_tail() {
            self.insert(ch)
        } else {
            let pos = self.position();
            self.0
                .contents_mut()
                .replace_range(pos..pos + 1, &ch.to_string());
            self.forward();
        }
    }

    pub fn erase(&mut self) {
        if !self.0.is_head() {
            self.backward();
            let pos = self.position();
            self.0.contents_mut().drain(pos..pos + 1);
        }
    }

    pub fn erase_all(&mut self) {
        *self = Self::default();
    }

    pub fn move_to_head(&mut self) {
        self.0.move_to_head()
    }

    pub fn move_to_tail(&mut self) {
        self.0.move_to_tail()
    }

    pub fn backward(&mut self) -> bool {
        self.0.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.0.forward()
    }
}

#[cfg(test)]
mod test {
    use crate::core::cursor::Cursor;

    use super::TextEditor;

    fn new_with_position(s: String, p: usize) -> TextEditor {
        TextEditor(Cursor::new_with_position(s, p))
    }

    mod masking {
        use crate::text_editor::test::new_with_position;

        #[test]
        fn test() {
            let txt = new_with_position(String::from("abcde "), 0);
            assert_eq!("***** ", txt.masking('*'))
        }
    }

    mod erase {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextEditor::default();
            assert_eq!(String::from(" "), txt.text());
            assert_eq!(0, txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("bc "),
                0, // indicate `b`.
            );
            txt.erase();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            let new = new_with_position(
                String::from("ab "),
                2, // indicate tail.
            );
            txt.erase();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_head() {
            let txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            assert_eq!(String::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod insert {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            let new = new_with_position(
                String::from("d "),
                1, // indicate tail.
            );
            txt.insert('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("adbc "),
                2, // indicate `b`.
            );
            txt.insert('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            let new = new_with_position(
                String::from("abcd "),
                4, // indicate tail.
            );
            txt.insert('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            let new = new_with_position(
                String::from("dabc "),
                1, // indicate `a`.
            );
            txt.insert('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }
    }

    mod overwrite {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            let new = new_with_position(
                String::from("d "),
                1, // indicate tail.
            );
            txt.overwrite('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("adc "),
                2, // indicate `c`.
            );
            txt.overwrite('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            let new = new_with_position(
                String::from("abcd "),
                4, // indicate tail.
            );
            txt.overwrite('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            let new = new_with_position(
                String::from("dbc "),
                1, // indicate `b`.
            );
            txt.overwrite('d');
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }
    }

    mod backward {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.backward();
            assert_eq!(String::from(" "), txt.text());
            assert_eq!(0, txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            txt.backward();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            let new = new_with_position(
                String::from("abc "),
                2, // indicate `c`.
            );
            txt.backward();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            txt.backward();
            assert_eq!(String::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod forward {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.forward();
            assert_eq!(String::from(" "), txt.text());
            assert_eq!(0, txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("abc "),
                2, // indicate `c`.
            );
            txt.forward();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            txt.forward();
            assert_eq!(String::from("abc "), txt.text());
            assert_eq!(3, txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            let new = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            txt.forward();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }
    }

    mod to_head {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_head();
            assert_eq!(String::from(" "), txt.text());
            assert_eq!(0, txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            txt.move_to_head();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            let new = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            txt.move_to_head();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            txt.move_to_head();
            assert_eq!(String::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod to_tail {
        use crate::text_editor::test::new_with_position;

        use super::super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_tail();
            assert_eq!(String::from(" "), txt.text());
            assert_eq!(0, txt.position());
        }

        #[test]
        fn test_at_non_edge() {
            let mut txt = new_with_position(
                String::from("abc "),
                1, // indicate `b`.
            );
            let new = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            txt.move_to_tail();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }

        #[test]
        fn test_at_tail() {
            let mut txt = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            txt.move_to_tail();
            assert_eq!(String::from("abc "), txt.text());
            assert_eq!(3, txt.position());
        }

        #[test]
        fn test_at_head() {
            let mut txt = new_with_position(
                String::from("abc "),
                0, // indicate `a`.
            );
            let new = new_with_position(
                String::from("abc "),
                3, // indicate tail.
            );
            txt.move_to_tail();
            assert_eq!(new.text(), txt.text());
            assert_eq!(new.position(), txt.position());
        }
    }
}
