use std::io;

use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal,
};

use promkit::{termutil, Result};

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut out = io::stdout();
    termutil::clear(&mut out)?;
    loop {
        {
            let label = format!("{:?} {:?}", terminal::size()?, cursor::position()?);
            crossterm::execute!(
                io::stdout(),
                cursor::SavePosition,
                cursor::MoveTo(0, 0),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print(label),
                cursor::RestorePosition,
            )?;
        }
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
            }) => {
                termutil::move_left(&mut out, 1)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                termutil::move_right(&mut out, 1)?;
            }
            _ => continue,
        };
    }
    terminal::disable_raw_mode()
}
