use unicode_width::UnicodeWidthStr;

use crate::{error::ScreenError, terminal::TerminalSize};

/// A builder for constructing a screen representation for testing purposes.
#[derive(Debug)]
pub struct Screen {
    size: TerminalSize,
    lines: Vec<Option<String>>,
}

impl Screen {
    // Create a new screen with the specified dimensions, initialized with blank lines.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: TerminalSize::new(rows, cols),
            lines: vec![None; rows as usize],
        }
    }

    // Set the content of a specific row, padding it to the terminal width.
    pub fn line(mut self, row: u16, content: &str) -> Result<Self, ScreenError> {
        if row >= self.size.rows {
            return Err(ScreenError::RowOutOfBounds {
                row,
                rows: self.size.rows,
            });
        }

        self.lines[row as usize] = Some(pad_to_cols(self.size.cols, content));
        Ok(self)
    }

    // Build the final screen representation as a vector of strings, filling in blank lines where necessary.
    pub fn build(self) -> Vec<String> {
        let blank = " ".repeat(self.size.cols as usize);
        self.lines
            .into_iter()
            .map(|line| line.unwrap_or_else(|| blank.clone()))
            .collect()
    }
}

pub fn pad_to_cols(cols: u16, content: &str) -> String {
    let width: usize = content.width();
    assert!(
        width <= cols as usize,
        "line width {width} exceeds terminal width {cols}"
    );

    let mut line = String::from(content);
    line.push_str(&" ".repeat(cols as usize - width));
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    mod screen {
        use super::*;

        #[test]
        fn build() {
            let screen = Screen::new(5, 3)
                .line(0, "Hi")
                .unwrap()
                .line(2, "Bye")
                .unwrap()
                .build();

            assert_eq!(
                screen,
                vec![
                    "Hi   ".to_string(),
                    "     ".to_string(),
                    "Bye  ".to_string(),
                ]
            );
        }
    }
}
