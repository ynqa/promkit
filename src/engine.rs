use std::{fmt, io::Write};

use crate::{
    crossterm::{
        cursor::{self, MoveTo},
        queue,
        execute,
        style::Print,
        terminal::{self, Clear, ClearType, ScrollUp},
    },
    error::{Error, Result},
};

/// Provides functionality for executing terminal commands
/// and managing terminal state.
///
/// The `Engine` struct encapsulates methods for interacting
/// with the terminal, such as moving the cursor,
/// clearing the screen, writing text, and more.
/// It leverages the `crossterm` crate to provide cross-platform
/// terminal operations.
#[derive(Clone)]
pub struct Engine<W: Write> {
    out: W,
}

impl<W: Write> Engine<W> {
    /// Constructs a new `Engine` with the specified output writer.
    ///
    /// # Arguments
    ///
    /// * `out` - The output writer where terminal commands will be executed.
    pub fn new(out: W) -> Self {
        Self { out }
    }

    /// Retrieves the current position of the cursor in the terminal.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the current cursor position as `(u16, u16)`,
    /// or an `Error` if unable to retrieve the position.
    pub fn position(&self) -> Result<(u16, u16)> {
        cursor::position().map_err(Error::from)
    }

    /// Retrieves the size of the terminal.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the terminal size as `(u16, u16)`,
    /// or an `Error` if unable to retrieve the size.
    pub fn size(&self) -> Result<(u16, u16)> {
        terminal::size().map_err(Error::from)
    }

    /// Clears the terminal screen from the cursor's current position downwards.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn clear_from_cursor_down(&mut self) -> Result {
        queue!(self.out, Clear(ClearType::FromCursorDown)).map_err(Error::from)
    }

    /// Clears the entire terminal screen
    /// and moves the cursor to the top-left position.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn clear(&mut self) -> Result {
        queue!(self.out, Clear(ClearType::All), MoveTo(0, 0)).map_err(Error::from)
    }

    /// Writes a string to the terminal.
    ///
    /// # Arguments
    ///
    /// * `string` - The string to be written.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn write<D: fmt::Display>(&mut self, string: D) -> Result {
        execute!(self.out, Print(string)).map_err(Error::from)
    }

    /// Moves the cursor to the specified position in the terminal.
    ///
    /// # Arguments
    ///
    /// * `pos` - The target position as `(u16, u16)`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn move_to(&mut self, pos: (u16, u16)) -> Result {
        queue!(self.out, MoveTo(pos.0, pos.1)).map_err(Error::from)
    }

    /// Scrolls the terminal content up by a specified number of lines.
    ///
    /// # Arguments
    ///
    /// * `times` - The number of lines to scroll up.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn scroll_up(&mut self, times: u16) -> Result {
        queue!(self.out, ScrollUp(times)).map_err(Error::from)
    }

    /// Moves the cursor to the next line in the terminal.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success of the operation,
    /// or an `Error` if it fails.
    pub fn move_to_next_line(&mut self) -> Result {
        queue!(self.out, cursor::MoveToNextLine(1)).map_err(Error::from)
    }
}

#[cfg(test)]
mod test {
    mod clear {
        use super::super::*;

        #[test]
        fn test() {
            let out = vec![];
            let mut engine = Engine::new(out);
            assert!(engine.clear().is_ok());
            assert_eq!(
                String::from_utf8(strip_ansi_escapes::strip(engine.out)).unwrap(),
                ""
            );
        }
    }

    mod write {
        use super::super::*;

        #[test]
        fn test() {
            let out = vec![];
            let mut engine = Engine::new(out);
            assert!(engine.write("abcde").is_ok());
            assert_eq!(
                String::from_utf8(strip_ansi_escapes::strip(engine.out)).unwrap(),
                "abcde"
            );
        }
    }
}
