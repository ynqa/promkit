use std::io;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        cursor::Hide,
        event::EnableMouseCapture,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    loop {
        if let Ok(event) = event::read() {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    break;
                }
                ev => {
                    execute!(
                        io::stdout(),
                        cursor::MoveToNextLine(1),
                        Print(format!("{:?}", ev)),
                    )?;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), cursor::Show, event::DisableMouseCapture)?;
    Ok(())
}
