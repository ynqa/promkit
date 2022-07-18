use std::cmp::Ordering;
use std::io;

use crossterm::{cursor, terminal};

use crate::{grapheme::Graphemes, Result};

/// All four sides of terminal.
pub enum Boundary {
    Top,
    Bottom,
    RightEdge,
    LeftEdge,
}

/// Clear all lines in screen, and its behavior is similar to
/// [clear(1)](https://man7.org/linux/man-pages/man1/clear.1.html).
pub fn clear<W: io::Write>(out: &mut W) -> Result<()> {
    crossterm::execute!(
        out,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
    )
}

/// Hide the cursor.
pub fn hide_cursor<W: io::Write>(out: &mut W) -> Result<()> {
    crossterm::execute!(out, cursor::Hide)
}

/// Show the cursor.
pub fn show_cursor<W: io::Write>(out: &mut W) -> Result<()> {
    crossterm::execute!(out, cursor::Show)
}

/// Move cursor to tail of the previous line.
pub fn move_up<W: io::Write>(out: &mut W) -> Result<()> {
    crossterm::execute!(
        out,
        cursor::MoveUp(1),
        cursor::MoveRight(terminal::size()?.0),
    )
}

/// Move cursor to head of the next line.
pub fn move_down<W: io::Write>(out: &mut W) -> Result<()> {
    let restored_position = cursor::position()?;
    crossterm::execute!(out, cursor::MoveToNextLine(1))?;
    if restored_position.1 == cursor::position()?.1
        && self::compare_cursor_position(Boundary::Bottom)? == Ordering::Equal
    {
        crossterm::execute!(out, terminal::ScrollUp(1))?;
    }
    Ok(())
}

/// Move cursor backward. At left-edge, it moves tail of the next line.
pub fn move_left<W: io::Write>(out: &mut W, n: u16) -> Result<()> {
    crossterm::execute!(out, cursor::Hide)?;
    for _ in 0..n {
        if self::compare_cursor_position(Boundary::LeftEdge)? == Ordering::Equal {
            self::move_up(out)?;
        } else {
            crossterm::execute!(out, cursor::MoveLeft(1))?;
        }
    }
    crossterm::execute!(out, cursor::Show)
}

/// Move cursor forward. At right-edge, it moves head of the next line.
pub fn move_right<W: io::Write>(out: &mut W, n: u16) -> Result<()> {
    crossterm::execute!(out, cursor::Hide)?;
    for _ in 0..n {
        if self::compare_cursor_position(Boundary::RightEdge)? == Ordering::Equal {
            self::move_down(out)?;
        } else {
            crossterm::execute!(out, cursor::MoveRight(1))?;
        }
    }
    crossterm::execute!(out, cursor::Show)
}

/// Move cursor to head of the current line.
pub fn move_head<W: io::Write>(out: &mut W) -> Result<()> {
    crossterm::execute!(out, cursor::MoveTo(0, cursor::position()?.1))
}

/// Return whether the current position of cursor is within or beyond the shape of terminal.
pub fn compare_cursor_position(boundary: Boundary) -> Result<Ordering> {
    match boundary {
        Boundary::Top | Boundary::Bottom => self::compare_position(cursor::position()?.1, boundary),
        Boundary::RightEdge | Boundary::LeftEdge => {
            self::compare_position(cursor::position()?.0, boundary)
        }
    }
}

/// Return whether the given position is within or beyond the shape of terminal.
pub fn compare_position(position: u16, boundary: Boundary) -> Result<Ordering> {
    match boundary {
        Boundary::Top => Ok(position.cmp(&0)),
        Boundary::Bottom => Ok((position + 1).cmp(&terminal::size()?.1)),
        Boundary::RightEdge => Ok((position + 1).cmp(&terminal::size()?.0)),
        Boundary::LeftEdge => Ok(position.cmp(&0)),
    }
}

/// Return the number of lines when the graphemes are rendered.
pub fn num_lines(graphemes: &Graphemes) -> Result<u16> {
    Ok(match graphemes.width() as u16 {
        0 => 0,
        v => v / terminal::size()?.0 + 1,
    })
}

#[test]
fn test_clear() {
    let mut out = vec![];
    assert!(self::clear(&mut out).is_ok());
    assert_eq!(
        String::from_utf8(strip_ansi_escapes::strip(out).unwrap()).unwrap(),
        ""
    );
}
