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
    preferred_column: Option<usize>,
}

/// A cursor position in newline-delimited text.
///
/// `column` is measured in terminal display cells rather than grapheme indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextPosition {
    pub row: usize,
    pub column: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            // Keep a trailing grapheme as the visible cursor.
            text: StyledGraphemes::from(" "),
            position: 0,
            preferred_column: None,
        }
    }
}

impl TextEditor {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let mut buf = s.as_ref().to_owned();
        buf.push(' ');
        let text = StyledGraphemes::from(buf);
        let position = text.len() - 1;
        Self {
            text,
            position,
            preferred_column: None,
        }
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

    /// Returns the cursor position in newline-delimited text.
    ///
    /// The row is a logical row separated by an explicit newline. The column is
    /// the terminal display width before the cursor on that row.
    pub fn logical_position(&self) -> TextPosition {
        let ranges = self.line_ranges();
        let (row, (start, _)) = ranges
            .iter()
            .copied()
            .enumerate()
            .find(|(_, (start, end))| self.position >= *start && self.position <= *end)
            .unwrap_or_else(|| {
                let row = ranges.len().saturating_sub(1);
                (row, ranges[row])
            });

        TextPosition {
            row,
            column: self.display_width_between(start, self.position),
        }
    }

    /// Resolves a logical text position to an absolute grapheme cursor index.
    ///
    /// Columns past the end of a row resolve to that row's end. A column inside
    /// a wide grapheme resolves to the position before that grapheme.
    pub fn position_at(&self, position: TextPosition) -> Option<usize> {
        let (start, end) = self.line_ranges().get(position.row).copied()?;
        Some(self.position_for_column(start, end, position.column))
    }

    /// Masks all characters except the cursor with the specified mask character.
    pub fn masking(&self, mask: char) -> StyledGraphemes {
        let cursor = self.text.len() - 1;
        self.text()
            .chars()
            .into_iter()
            .enumerate()
            .map(|(i, c)| StyledGrapheme::from(if i == cursor || c == '\n' { c } else { mask }))
            .collect::<StyledGraphemes>()
    }

    /// Replaces the current text with new text and positions the cursor at the end.
    pub fn replace(&mut self, new: &str) {
        *self = Self::new(new);
    }

    /// Inserts a character at the current cursor position.
    pub fn insert(&mut self, ch: char) {
        self.preferred_column = None;
        let pos = self.position();
        self.text.insert(pos, StyledGrapheme::from(ch));
        self.forward();
    }

    /// Inserts a newline at the current cursor position.
    pub fn insert_newline(&mut self) {
        self.insert('\n');
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
            self.preferred_column = None;
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
            self.preferred_column = None;
            self.backward();
            let pos = self.position();
            self.text.drain(pos..pos + 1);
        }
    }

    /// Erases the grapheme at the cursor position.
    ///
    /// Erasing a newline joins its surrounding logical rows. The trailing
    /// cursor grapheme is never removed.
    pub fn erase_forward(&mut self) {
        if self.position < self.text.len() - 1 {
            self.preferred_column = None;
            self.text.drain(self.position..self.position + 1);
        }
    }

    /// Clears all text and resets the editor to its default state.
    pub fn erase_all(&mut self) {
        *self = Self::default();
    }

    /// Erases the text from the current cursor position to the specified position,
    /// considering whether pos is greater or smaller than the current position.
    fn erase_to_position(&mut self, pos: usize) {
        self.preferred_column = None;
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
        self.preferred_column = None;
    }

    /// Moves the cursor to the end of the text.
    pub fn move_to_tail(&mut self) {
        self.position = self.text.len() - 1;
        self.preferred_column = None;
    }

    /// Moves the cursor to the beginning of its current logical row.
    pub fn move_to_line_head(&mut self) {
        let position = self.logical_position();
        if let Some((start, _)) = self.line_ranges().get(position.row) {
            self.position = *start;
            self.preferred_column = None;
        }
    }

    /// Moves the cursor to the end of its current logical row.
    pub fn move_to_line_tail(&mut self) {
        let position = self.logical_position();
        if let Some((_, end)) = self.line_ranges().get(position.row) {
            self.position = *end;
            self.preferred_column = None;
        }
    }

    /// Moves the cursor to the previous logical row.
    ///
    /// Repeated vertical movement preserves the original display column and
    /// clamps only while passing through shorter rows.
    pub fn move_up(&mut self) -> bool {
        self.move_vertical(-1)
    }

    /// Moves the cursor to the next logical row.
    ///
    /// Repeated vertical movement preserves the original display column and
    /// clamps only while passing through shorter rows.
    pub fn move_down(&mut self) -> bool {
        self.move_vertical(1)
    }

    /// Moves the cursor to a character by index.
    pub fn move_to(&mut self, position: usize) -> bool {
        if position < self.text.len() {
            self.position = position;
            self.preferred_column = None;
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

    fn move_vertical(&mut self, row_delta: isize) -> bool {
        let ranges = self.line_ranges();
        let current = self.logical_position();
        let Some(target_row) = current.row.checked_add_signed(row_delta) else {
            return false;
        };
        let Some((start, end)) = ranges.get(target_row).copied() else {
            return false;
        };

        let column = self.preferred_column.unwrap_or(current.column);
        self.position = self.position_for_column(start, end, column);
        self.preferred_column = Some(column);
        true
    }

    fn line_ranges(&self) -> Vec<(usize, usize)> {
        let content_end = self.text.len().saturating_sub(1);
        let mut ranges = Vec::new();
        let mut start = 0;

        for (index, grapheme) in self.text.iter().take(content_end).enumerate() {
            if grapheme.character() == '\n' {
                ranges.push((start, index));
                start = index + 1;
            }
        }

        ranges.push((start, content_end));
        ranges
    }

    fn display_width_between(&self, start: usize, end: usize) -> usize {
        self.text
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(StyledGrapheme::width)
            .sum()
    }

    fn position_for_column(&self, start: usize, end: usize, column: usize) -> usize {
        let mut current_column = 0;

        for (offset, grapheme) in self
            .text
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .enumerate()
        {
            let next_column = current_column + grapheme.width();
            if column < next_column {
                return start + offset;
            }
            current_column = next_column;
        }

        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod text_editor {
        use super::*;

        fn new_with_position(s: String, p: usize) -> TextEditor {
            let text = StyledGraphemes::from(s);
            TextEditor {
                position: p.min(text.len().saturating_sub(1)),
                text,
                preferred_column: None,
            }
        }

        mod new {
            use super::*;

            #[test]
            fn starts_at_the_trailing_cursor() {
                let texteditor = TextEditor::new("abc");

                assert_eq!(texteditor.position(), 3);
            }
        }

        mod move_to {
            use super::*;

            #[test]
            fn preserves_the_position_when_the_target_is_out_of_bounds() {
                let mut texteditor = TextEditor::new("abc");

                assert!(texteditor.move_to(1));
                assert_eq!(texteditor.position(), 1);
                assert!(!texteditor.move_to(4));
                assert_eq!(texteditor.position(), 1);
            }
        }

        mod shift {
            use super::*;

            #[test]
            fn preserves_the_position_when_the_target_is_out_of_bounds() {
                let mut texteditor = TextEditor::new("abc");
                assert!(texteditor.move_to(1));

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
            fn replaces_non_whitespace_characters_with_the_mask() {
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
            fn empty_editor_is_unchanged() {
                let txt = TextEditor::default();
                assert_eq!(StyledGraphemes::from(" "), txt.text());
                assert_eq!(0, txt.position());
            }

            #[test]
            fn removes_the_character_before_the_cursor() {
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
            fn removes_the_last_character_at_the_tail() {
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
            fn head_is_unchanged() {
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
            fn finds_the_previous_word_boundary() {
                let mut txt = new_with_position(String::from("koko momo jojo "), 11); // indicate `o`.
                assert_eq!(10, txt.find_previous_nearest_index(&HashSet::from([' '])));
                txt.move_to(10);
                assert_eq!(5, txt.find_previous_nearest_index(&HashSet::from([' '])));
            }

            #[test]
            fn returns_the_head_when_no_boundary_exists() {
                let txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
                assert_eq!(0, txt.find_previous_nearest_index(&HashSet::from(['z'])));
            }
        }

        mod find_next_nearest_index {
            use super::*;

            use std::collections::HashSet;

            #[test]
            fn finds_the_next_word_boundary() {
                let mut txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
                assert_eq!(10, txt.find_next_nearest_index(&HashSet::from([' '])));
                txt.move_to(10);
                assert_eq!(14, txt.find_next_nearest_index(&HashSet::from([' '])));
            }

            #[test]
            fn returns_the_tail_when_no_boundary_exists() {
                let txt = new_with_position(String::from("koko momo jojo "), 7); // indicate `m`.
                assert_eq!(14, txt.find_next_nearest_index(&HashSet::from(['z'])));
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn inserts_into_an_empty_editor() {
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
            fn inserts_before_the_cursor() {
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
            fn appends_at_the_tail() {
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
            fn prepends_at_the_head() {
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

        mod logical_position {
            use super::*;

            #[test]
            fn uses_display_columns() {
                let mut txt = TextEditor::new("ab\n界c");

                assert_eq!(TextPosition { row: 1, column: 3 }, txt.logical_position());

                assert!(txt.move_to(3)); // Before `界`.
                assert_eq!(TextPosition { row: 1, column: 0 }, txt.logical_position());
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

        mod insert_newline {
            use super::*;

            #[test]
            fn inserts_at_the_cursor() {
                let mut txt = TextEditor::new("abcd");
                assert!(txt.move_to(2));

                txt.insert_newline();

                assert_eq!("ab\ncd", txt.text_without_cursor().to_string());
                assert_eq!(3, txt.position());
                assert_eq!(TextPosition { row: 1, column: 0 }, txt.logical_position());
            }
        }

        mod move_down {
            use super::*;

            #[test]
            fn preserves_the_preferred_display_column() {
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
            fn uses_wide_character_display_widths() {
                let mut txt = TextEditor::new("界a\n123");
                assert!(txt.move_to(1)); // Display column 2 after `界`.

                assert!(txt.move_down());

                assert_eq!(5, txt.position());
                assert_eq!(TextPosition { row: 1, column: 2 }, txt.logical_position());
            }

            #[test]
            fn stops_at_the_document_tail() {
                let mut txt = TextEditor::new("ab\ncd");
                txt.move_to_tail();

                assert!(!txt.move_down());
            }
        }

        mod move_up {
            use super::*;

            #[test]
            fn stops_at_the_document_head() {
                let mut txt = TextEditor::new("ab\ncd");
                txt.move_to_head();

                assert!(!txt.move_up());
                assert_eq!(0, txt.position());

                assert!(txt.move_to(3));
                assert!(txt.move_up());
                assert_eq!(0, txt.position());
                assert!(!txt.move_up());
            }
        }

        mod move_to_line_head {
            use super::*;

            #[test]
            fn moves_to_the_current_line_head() {
                let mut txt = TextEditor::new("ab\ncd");
                assert!(txt.move_to(4)); // Before `d`.

                txt.move_to_line_head();
                assert_eq!(3, txt.position());
            }
        }

        mod move_to_line_tail {
            use super::*;

            #[test]
            fn moves_to_the_current_line_tail() {
                let mut txt = TextEditor::new("ab\ncd");
                assert!(txt.move_to(4)); // Before `d`.

                txt.move_to_line_tail();
                assert_eq!(5, txt.position());
            }
        }

        mod erase_forward {
            use super::*;

            #[test]
            fn erases_a_newline_and_joins_lines() {
                let mut txt = TextEditor::new("ab\ncd");
                assert!(txt.move_to(2)); // Before the newline.

                txt.erase_forward();

                assert_eq!("abcd", txt.text_without_cursor().to_string());
                assert_eq!(2, txt.position());
                assert_eq!(TextPosition { row: 0, column: 2 }, txt.logical_position());
            }
        }

        mod overwrite {
            use super::*;

            #[test]
            fn inserts_into_an_empty_editor() {
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
            fn replaces_the_character_at_the_cursor() {
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
            fn appends_at_the_tail() {
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
            fn replaces_the_first_character_at_the_head() {
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
            fn empty_editor_is_unchanged() {
                let mut txt = TextEditor::default();
                txt.backward();
                assert_eq!(StyledGraphemes::from(" "), txt.text());
                assert_eq!(0, txt.position());
            }

            #[test]
            fn moves_back_one_character() {
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
            fn moves_back_from_the_tail() {
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
            fn head_is_unchanged() {
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
            fn empty_editor_is_unchanged() {
                let mut txt = TextEditor::default();
                txt.forward();
                assert_eq!(StyledGraphemes::from(" "), txt.text());
                assert_eq!(0, txt.position());
            }

            #[test]
            fn moves_forward_one_character() {
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
            fn tail_is_unchanged() {
                let mut txt = new_with_position(
                    String::from("abc "),
                    3, // indicate tail.
                );
                txt.forward();
                assert_eq!(StyledGraphemes::from("abc "), txt.text());
                assert_eq!(3, txt.position());
            }

            #[test]
            fn moves_forward_from_the_head() {
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

        mod move_to_head {
            use super::*;

            #[test]
            fn empty_editor_is_unchanged() {
                let mut txt = TextEditor::default();
                txt.move_to_head();
                assert_eq!(StyledGraphemes::from(" "), txt.text());
                assert_eq!(0, txt.position());
            }

            #[test]
            fn moves_to_the_head() {
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
            fn moves_from_the_tail_to_the_head() {
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
            fn head_is_unchanged() {
                let mut txt = new_with_position(
                    String::from("abc "),
                    0, // indicate `a`.
                );
                txt.move_to_head();
                assert_eq!(StyledGraphemes::from("abc "), txt.text());
                assert_eq!(0, txt.position());
            }
        }

        mod move_to_tail {
            use super::*;

            #[test]
            fn empty_editor_is_unchanged() {
                let mut txt = TextEditor::default();
                txt.move_to_tail();
                assert_eq!(StyledGraphemes::from(" "), txt.text());
                assert_eq!(0, txt.position());
            }

            #[test]
            fn moves_to_the_tail() {
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
            fn tail_is_unchanged() {
                let mut txt = new_with_position(
                    String::from("abc "),
                    3, // indicate tail.
                );
                txt.move_to_tail();
                assert_eq!(StyledGraphemes::from("abc "), txt.text());
                assert_eq!(3, txt.position());
            }

            #[test]
            fn moves_from_the_head_to_the_tail() {
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
}
