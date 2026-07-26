use std::collections::HashSet;

use promkit_core::grapheme::{StyledGrapheme, StyledGraphemes};

/// Edit mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

/// A text editor that supports basic editing operations
/// such as insert, delete, and overwrite.
#[derive(Clone)]
pub struct TextEditor {
    text: StyledGraphemes,
    position: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            // Keep a trailing grapheme as the visible cursor.
            text: StyledGraphemes::from(" "),
            position: 0,
        }
    }
}

impl TextEditor {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let mut buf = s.as_ref().to_owned();
        buf.push(' ');
        let text = StyledGraphemes::from(buf);
        let position = text.len() - 1;
        Self { text, position }
    }

    /// Returns the current text including the cursor.
    pub fn text(&self) -> StyledGraphemes {
        self.text.clone()
    }

    /// Returns the text without the cursor.
    pub fn text_without_cursor(&self) -> StyledGraphemes {
        let mut ret = self.text();
        ret.pop_back();
        ret
    }

    /// Returns the current position of the cursor within the text.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Masks all characters except the cursor with the specified mask character.
    pub fn masking(&self, mask: char) -> StyledGraphemes {
        self.text()
            .chars()
            .into_iter()
            .enumerate()
            .map(|(i, c)| StyledGrapheme::from(if i == self.text().len() - 1 { c } else { mask }))
            .collect::<StyledGraphemes>()
    }

    /// Replaces the current text with new text and positions the cursor at the end.
    pub fn replace(&mut self, new: &str) {
        *self = Self::new(new);
    }

    /// Inserts a character at the current cursor position.
    pub fn insert(&mut self, ch: char) {
        let pos = self.position();
        self.text.insert(pos, StyledGrapheme::from(ch));
        self.forward();
    }

    pub fn insert_chars(&mut self, vch: &Vec<char>) {
        for ch in vch {
            self.insert(*ch);
        }
    }

    /// Overwrites the character at the current cursor position with the specified character.
    pub fn overwrite(&mut self, ch: char) {
        if self.position == self.text.len() - 1 {
            self.insert(ch)
        } else {
            let pos = self.position();
            self.text.replace_range(pos..pos + 1, ch.to_string());
            self.forward();
        }
    }

    pub fn overwrite_chars(&mut self, vch: &Vec<char>) {
        for ch in vch {
            self.overwrite(*ch);
        }
    }

    /// Erases the character before the cursor position.
    pub fn erase(&mut self) {
        if self.position > 0 {
            self.backward();
            let pos = self.position();
            self.text.drain(pos..pos + 1);
        }
    }

    /// Clears all text and resets the editor to its default state.
    pub fn erase_all(&mut self) {
        *self = Self::default();
    }

    /// Erases the text from the current cursor position to the specified position,
    /// considering whether pos is greater or smaller than the current position.
    fn erase_to_position(&mut self, pos: usize) {
        let current_pos = self.position();
        if pos > current_pos {
            self.text.drain(current_pos..pos);
        } else {
            self.text.drain(pos..current_pos);
            self.move_to(pos);
        }
    }

    /// Finds the nearest previous index of any character in `word_break_chars` from the cursor position.
    fn find_previous_nearest_index(&self, word_break_chars: &HashSet<char>) -> usize {
        let current_position = self.position();
        self.text()
            .chars()
            .iter()
            .enumerate()
            .filter(|&(i, _)| i < current_position.saturating_sub(1))
            .rev()
            .find(|&(_, c)| word_break_chars.contains(c))
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
    }

    /// Erases the text from the current cursor position to the nearest previous character in `word_break_chars`.
    pub fn erase_to_previous_nearest(&mut self, word_break_chars: &HashSet<char>) {
        let pos = self.find_previous_nearest_index(word_break_chars);
        self.erase_to_position(pos);
    }

    /// Moves the cursor to the nearest previous character in `word_break_chars`.
    pub fn move_to_previous_nearest(&mut self, word_break_chars: &HashSet<char>) {
        let pos = self.find_previous_nearest_index(word_break_chars);
        self.move_to(pos);
    }

    /// Finds the nearest next index of any character in `word_break_chars` from the cursor position.
    fn find_next_nearest_index(&self, word_break_chars: &HashSet<char>) -> usize {
        let current_position = self.position();
        self.text()
            .chars()
            .iter()
            .enumerate()
            .filter(|&(i, _)| i > current_position)
            .find(|&(_, c)| word_break_chars.contains(c))
            .map(|(i, _)| {
                if i < self.text.len() - 1 {
                    i + 1
                } else {
                    self.text.len() - 1
                }
            })
            .unwrap_or(self.text.len() - 1)
    }

    /// Erases the text from the current cursor position to the nearest next character in `word_break_chars`.
    pub fn erase_to_next_nearest(&mut self, word_break_chars: &HashSet<char>) {
        let pos = self.find_next_nearest_index(word_break_chars);
        self.erase_to_position(pos);
    }

    /// Moves the cursor to the nearest next character in `word_break_chars`.
    pub fn move_to_next_nearest(&mut self, word_break_chars: &HashSet<char>) {
        let pos = self.find_next_nearest_index(word_break_chars);
        self.move_to(pos);
    }

    /// Moves the cursor to the beginning of the text.
    pub fn move_to_head(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor to the end of the text.
    pub fn move_to_tail(&mut self) {
        self.position = self.text.len() - 1;
    }

    /// Moves the cursor to a character by index.
    pub fn move_to(&mut self, position: usize) -> bool {
        if position < self.text.len() {
            self.position = position;
            true
        } else {
            false
        }
    }

    pub fn shift(&mut self, backward: usize, forward: usize) -> bool {
        let Some(position) = self
            .position
            .checked_sub(backward)
            .and_then(|position| position.checked_add(forward))
        else {
            return false;
        };

        self.move_to(position)
    }

    /// Moves the cursor one position backward, if possible.
    pub fn backward(&mut self) -> bool {
        self.shift(1, 0)
    }

    /// Moves the cursor one position forward, if possible.
    pub fn forward(&mut self) -> bool {
        self.shift(0, 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_with_position(s: String, p: usize) -> TextEditor {
        let text = StyledGraphemes::from(s);
        TextEditor {
            position: p.min(text.len().saturating_sub(1)),
            text,
        }
    }

    mod position {
        use super::*;

        #[test]
        fn starts_at_the_trailing_cursor() {
            let texteditor = TextEditor::new("abc");

            assert_eq!(texteditor.position(), 3);
        }

        #[test]
        fn direct_and_relative_moves_preserve_position_on_failure() {
            let mut texteditor = TextEditor::new("abc");

            assert!(texteditor.move_to(1));
            assert_eq!(texteditor.position(), 1);
            assert!(!texteditor.move_to(4));
            assert_eq!(texteditor.position(), 1);

            assert!(texteditor.shift(0, 2));
            assert_eq!(texteditor.position(), 3);
            assert!(!texteditor.shift(0, 1));
            assert_eq!(texteditor.position(), 3);
            assert!(!texteditor.shift(4, 0));
            assert_eq!(texteditor.position(), 3);
        }
    }

    mod masking {
        use super::*;

        #[test]
        fn test() {
            let txt = new_with_position(String::from("abcde "), 0);
            assert_eq!(StyledGraphemes::from("***** "), txt.masking('*'))
        }

        #[test]
        fn preserves_newlines_in_multiline_text() {
            let txt = TextEditor::new("ab\nc");

            assert_eq!(StyledGraphemes::from("**\n* "), txt.masking('*'));
        }
    }

    mod erase {
        use super::*;

        #[test]
        fn test_for_empty() {
            let txt = TextEditor::default();
            assert_eq!(StyledGraphemes::from(" "), txt.text());
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
            assert_eq!(StyledGraphemes::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod find_previous_nearest_index {
        use super::*;

        use std::collections::HashSet;

        #[test]
        fn test() {
            let mut txt = new_with_position(String::from("koko momo jojo "), 11); // indicate `o`.
            assert_eq!(10, txt.find_previous_nearest_index(&HashSet::from([' '])));
            txt.move_to(10);
            assert_eq!(5, txt.find_previous_nearest_index(&HashSet::from([' '])));
        }

        #[test]
        fn test_with_no_target() {
            let txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
            assert_eq!(0, txt.find_previous_nearest_index(&HashSet::from(['z'])));
        }
    }

    mod find_next_nearest_index {
        use super::*;

        use std::collections::HashSet;

        #[test]
        fn test() {
            let mut txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
            assert_eq!(10, txt.find_next_nearest_index(&HashSet::from([' '])));
            txt.move_to(10);
            assert_eq!(14, txt.find_next_nearest_index(&HashSet::from([' '])));
        }

        #[test]
        fn test_with_no_target() {
            let txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
            assert_eq!(14, txt.find_next_nearest_index(&HashSet::from(['z'])));
        }
    }

    mod insert {
        use super::*;

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

    mod multiline {
        use super::*;

        #[test]
        fn reports_logical_position_using_display_columns() {
            let mut txt = TextEditor::new("ab\n界c");

            assert_eq!(TextPosition { row: 1, column: 3 }, txt.logical_position());

            assert!(txt.move_to(3)); // Before `界`.
            assert_eq!(TextPosition { row: 1, column: 0 }, txt.logical_position());
        }

        #[test]
        fn inserts_a_newline_at_the_cursor() {
            let mut txt = TextEditor::new("abcd");
            assert!(txt.move_to(2));

            txt.insert_newline();

            assert_eq!("ab\ncd", txt.text_without_cursor().to_string());
            assert_eq!(3, txt.position());
            assert_eq!(TextPosition { row: 1, column: 0 }, txt.logical_position());
        }

        #[test]
        fn moves_vertically_and_preserves_the_preferred_display_column() {
            let mut txt = TextEditor::new("abcdef\nxy\n123456");
            assert!(txt.move_to(6)); // End of the first line.

            assert!(txt.move_down());
            assert_eq!(9, txt.position()); // End of the shorter second line.
            assert_eq!(TextPosition { row: 1, column: 2 }, txt.logical_position());

            assert!(txt.move_down());
            assert_eq!(16, txt.position()); // Restore column 6 on the third line.
            assert_eq!(TextPosition { row: 2, column: 6 }, txt.logical_position());

            assert!(txt.move_up());
            assert_eq!(9, txt.position());
            assert!(txt.move_up());
            assert_eq!(6, txt.position());
        }

        #[test]
        fn moves_vertically_using_wide_character_display_widths() {
            let mut txt = TextEditor::new("界a\n123");
            assert!(txt.move_to(1)); // Display column 2 after `界`.

            assert!(txt.move_down());

            assert_eq!(5, txt.position());
            assert_eq!(TextPosition { row: 1, column: 2 }, txt.logical_position());
        }

        #[test]
        fn stops_vertical_movement_at_document_boundaries() {
            let mut txt = TextEditor::new("ab\ncd");
            txt.move_to_head();

            assert!(!txt.move_up());
            assert_eq!(0, txt.position());

            assert!(txt.move_to(3));
            assert!(txt.move_up());
            assert_eq!(0, txt.position());
            assert!(!txt.move_up());

            txt.move_to_tail();
            assert!(!txt.move_down());
        }

        #[test]
        fn moves_to_the_current_line_boundaries() {
            let mut txt = TextEditor::new("ab\ncd");
            assert!(txt.move_to(4)); // Before `d`.

            txt.move_to_line_head();
            assert_eq!(3, txt.position());

            txt.move_to_line_tail();
            assert_eq!(5, txt.position());
        }

        #[test]
        fn erases_a_newline_forward_and_joins_lines() {
            let mut txt = TextEditor::new("ab\ncd");
            assert!(txt.move_to(2)); // Before the newline.

            txt.erase_forward();

            assert_eq!("abcd", txt.text_without_cursor().to_string());
            assert_eq!(2, txt.position());
            assert_eq!(TextPosition { row: 0, column: 2 }, txt.logical_position());
        }

        #[test]
        fn handles_empty_lines_and_a_trailing_newline() {
            let mut txt = TextEditor::new("a\n\n");

            assert_eq!(TextPosition { row: 2, column: 0 }, txt.logical_position());
            assert!(txt.move_up());
            assert_eq!(TextPosition { row: 1, column: 0 }, txt.logical_position());
            assert!(txt.move_up());
            assert_eq!(TextPosition { row: 0, column: 0 }, txt.logical_position());
        }
    }

    mod overwrite {
        use super::*;

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
        use super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.backward();
            assert_eq!(StyledGraphemes::from(" "), txt.text());
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
            assert_eq!(StyledGraphemes::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod forward {
        use super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.forward();
            assert_eq!(StyledGraphemes::from(" "), txt.text());
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
            assert_eq!(StyledGraphemes::from("abc "), txt.text());
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
        use super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_head();
            assert_eq!(StyledGraphemes::from(" "), txt.text());
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
            assert_eq!(StyledGraphemes::from("abc "), txt.text());
            assert_eq!(0, txt.position());
        }
    }

    mod to_tail {
        use super::*;

        #[test]
        fn test_for_empty() {
            let mut txt = TextEditor::default();
            txt.move_to_tail();
            assert_eq!(StyledGraphemes::from(" "), txt.text());
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
            assert_eq!(StyledGraphemes::from("abc "), txt.text());
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
