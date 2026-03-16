use promkit_core::grapheme::StyledGraphemes;

use crate::cursor::Cursor;

#[derive(Clone)]
pub struct Text(Cursor<Vec<StyledGraphemes>>);

impl Default for Text {
    fn default() -> Self {
        Self(Cursor::new(vec![], 0, false))
    }
}

impl<T: AsRef<str>> From<T> for Text {
    fn from(text: T) -> Self {
        let value = text.as_ref();
        let lines: Vec<StyledGraphemes> = if value.is_empty() {
            Vec::new()
        } else {
            value
                .split('\n')
                // Replace empty lines with null character to
                // prevent them from being ignored at `style::Print`
                .map(|line| if line.is_empty() { "\0" } else { line })
                .map(StyledGraphemes::from)
                .collect()
        };
        Self(Cursor::new(lines, 0, false))
    }
}

impl Text {
    /// Creates a new `Text` from styled graphemes without parsing a string.
    /// Useful when the caller already has styled content prepared.
    pub fn from_styled_graphemes(lines: Vec<StyledGraphemes>) -> Self {
        Self(Cursor::new(lines, 0, false))
    }

    /// Replaces the contents with new contents and adjusts the position if necessary.
    pub fn replace_contents(&mut self, text: Vec<StyledGraphemes>) {
        self.0.replace_contents(text);
    }

    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<StyledGraphemes> {
        self.0.contents()
    }

    /// Returns the current position of the cursor within the listbox.
    pub fn position(&self) -> usize {
        self.0.position()
    }

    /// Moves the cursor backward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.0.backward()
    }

    /// Moves the cursor forward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.0.forward()
    }
}

#[cfg(test)]
mod tests {
    use super::Text;

    #[test]
    fn empty_input_creates_no_lines() {
        let text = Text::from("");
        assert!(text.items().is_empty());
    }

    #[test]
    fn explicit_empty_lines_are_preserved() {
        let text = Text::from("a\n\nb");
        assert_eq!(text.items().len(), 3);
        assert_eq!(text.items()[1].chars(), vec!['\0']);
    }
}
