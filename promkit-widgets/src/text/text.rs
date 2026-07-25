use promkit_core::grapheme::StyledGraphemes;

#[derive(Clone, Default)]
pub struct Text {
    lines: Vec<StyledGraphemes>,
    current_line: Option<usize>,
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
        let current_line = (!lines.is_empty()).then_some(0);
        Self {
            lines,
            current_line,
        }
    }
}

impl Text {
    /// Creates a new `Text` from styled graphemes without parsing a string.
    /// Useful when the caller already has styled content prepared.
    pub fn from_styled_graphemes(lines: Vec<StyledGraphemes>) -> Self {
        let current_line = (!lines.is_empty()).then_some(0);
        Self {
            lines,
            current_line,
        }
    }

    /// Replaces the contents with new contents and adjusts the position if necessary.
    pub fn replace_contents(&mut self, lines: Vec<StyledGraphemes>) {
        self.current_line = if lines.is_empty() {
            None
        } else {
            Some(self.current_line.unwrap_or_default().min(lines.len() - 1))
        };
        self.lines = lines;
    }

    /// Returns a reference to the rendered lines.
    pub fn items(&self) -> &Vec<StyledGraphemes> {
        &self.lines
    }

    /// Returns the current line, defaulting to zero when empty.
    ///
    /// Use [`Self::current_line`] when the empty state needs to be distinguished.
    pub fn position(&self) -> usize {
        self.current_line.unwrap_or_default()
    }

    /// Returns the current line, or `None` when there are no lines.
    pub fn current_line(&self) -> Option<usize> {
        self.current_line
    }

    /// Moves to the previous line, if possible.
    pub fn backward(&mut self) -> bool {
        let Some(position) = self.current_line.filter(|position| *position > 0) else {
            return false;
        };
        self.current_line = Some(position - 1);
        true
    }

    /// Moves to the next line, if possible.
    pub fn forward(&mut self) -> bool {
        let Some(position) = self
            .current_line
            .filter(|position| position.saturating_add(1) < self.lines.len())
        else {
            return false;
        };
        self.current_line = Some(position + 1);
        true
    }

    /// Moves to a line by index.
    pub fn move_to(&mut self, position: usize) -> bool {
        if position < self.lines.len() {
            self.current_line = Some(position);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use promkit_core::grapheme::StyledGraphemes;

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

    #[test]
    fn replacing_empty_text_restores_a_current_line() {
        let mut text = Text::default();
        text.replace_contents(vec![StyledGraphemes::from("first")]);

        assert_eq!(text.current_line(), Some(0));
    }

    #[test]
    fn navigation_stops_at_the_text_boundaries() {
        let mut text = Text::from("first\nsecond");

        assert_eq!(text.current_line(), Some(0));
        assert!(!text.backward());
        assert!(text.forward());
        assert_eq!(text.current_line(), Some(1));
        assert!(!text.forward());

        assert!(text.move_to(0));
        assert_eq!(text.current_line(), Some(0));
        assert!(!text.move_to(2));
        assert_eq!(text.current_line(), Some(0));
    }
}
