use std::{fmt, io::Write};

use crate::{
    crossterm::{
        cursor::{self, MoveTo},
        execute,
        style::Print,
        terminal::{self, Clear, ClearType, ScrollUp},
    },
    error::{Error, Result},
};

#[derive(Clone)]
pub struct Engine<W: Write> {
    out: W,
}

impl<W: Write> Engine<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn position(&self) -> Result<(u16, u16)> {
        cursor::position().map_err(|err| Error::from(err))
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        terminal::size().map_err(|err| Error::from(err))
    }

    pub fn clear_from_cursor_down(&mut self) -> Result {
        execute!(self.out, Clear(ClearType::FromCursorDown)).map_err(|err| Error::from(err))
    }

    pub fn clear(&mut self) -> Result {
        execute!(self.out, Clear(ClearType::All), MoveTo(0, 0)).map_err(|err| Error::from(err))
    }

    pub fn write<D: fmt::Display>(&mut self, string: D) -> Result {
        execute!(self.out, Print(string)).map_err(|err| Error::from(err))
    }

    pub fn move_to(&mut self, pos: (u16, u16)) -> Result {
        execute!(self.out, MoveTo(pos.0, pos.1)).map_err(|err| Error::from(err))
    }

    pub fn is_bottom(&self) -> Result<bool> {
        Ok(cursor::position()?.1 + 1 == terminal::size()?.1)
    }

    pub fn scroll_up(&mut self, times: u16) -> Result {
        execute!(self.out, ScrollUp(times)).map_err(|err| Error::from(err))
    }

    pub fn move_to_next_line(&mut self) -> Result {
        execute!(self.out, cursor::MoveToNextLine(1)).map_err(|err| Error::from(err))
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
