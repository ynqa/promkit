use std::{fmt, io::Write};

use anyhow::Result;

use crate::crossterm::{
    cursor::{self, MoveTo},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, ScrollUp},
};

#[derive(Clone)]
pub struct Engine<W: Write> {
    out: W,
}

impl<W: Write> Engine<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn position(&self) -> Result<(u16, u16), std::io::Error> {
        cursor::position()
    }

    pub fn size(&self) -> Result<(u16, u16), std::io::Error> {
        terminal::size()
    }

    pub fn clear_from_cursor_down(&mut self) -> Result<(), std::io::Error> {
        execute!(self.out, Clear(ClearType::FromCursorDown))
    }

    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        execute!(self.out, Clear(ClearType::All), MoveTo(0, 0))
    }

    pub fn write<D: fmt::Display>(&mut self, string: D) -> Result<(), std::io::Error> {
        execute!(self.out, Print(format!("{}", string)))
    }

    pub fn move_to(&mut self, pos: (u16, u16)) -> Result<(), std::io::Error> {
        execute!(self.out, MoveTo(pos.0, pos.1))
    }

    pub fn is_bottom(&self) -> Result<bool> {
        Ok(cursor::position()?.1 + 1 == terminal::size()?.1)
    }

    pub fn scroll_up(&mut self, times: u16) -> Result<(), std::io::Error> {
        execute!(self.out, ScrollUp(times))
    }

    pub fn move_to_next_line(&mut self) -> Result<(), std::io::Error> {
        execute!(self.out, cursor::MoveToNextLine(1))
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
