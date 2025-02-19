use std::io::Write;

use crate::{
    crossterm::{cursor, style, terminal},
    pane::Pane,
};

pub struct Terminal<'a, W: Write> {
    /// The current cursor position within the terminal.
    pub position: (u16, u16),
    pub writer: &'a mut W,
}

impl<'a, W: Write> Terminal<'a, W> {
    pub fn on_resize(&mut self) -> anyhow::Result<()> {
        self.position = (0, 0);
        crossterm::execute!(
            self.writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge),
        )?;
        Ok(())
    }

    pub fn start_session(panes: &[Pane], writer: &'a mut W) -> anyhow::Result<Self> {
        let position = cursor::position()?;
        let size = terminal::size()?;

        // If the cursor is not at the beginning of a line (position.0 != 0),
        // there are two scenarios to consider:
        // 1. If the cursor is also at the last line of the terminal (size.1 == position.1 + 1),
        //    the terminal is scrolled up by one line to make room.
        // 2. Regardless of whether a scroll occurred, move the cursor to the beginning of the next line
        //    to ensure the next output starts correctly.
        if position.0 != 0 {
            if size.1 == position.1 + 1 {
                crossterm::queue!(writer, terminal::ScrollUp(1))?;
            }
            crossterm::queue!(writer, cursor::MoveToNextLine(1))?;
        }

        // Calculate the total number of rows required by all panes.
        let lines = panes
            .iter()
            .map(|pane| pane.visible_row_count())
            .sum::<usize>();
        // If the cursor is at the last line of the terminal,
        // scroll up by the number of lines required by all panes,
        // and then move the cursor up by the same number of lines
        // to maintain its relative position.
        if size.1 == position.1 + 1 {
            crossterm::queue!(
                writer,
                terminal::ScrollUp(lines as u16),
                cursor::MoveToPreviousLine(lines as u16),
            )?;
        }

        writer.flush()?;

        Ok(Self {
            position: cursor::position()?,
            writer,
        })
    }

    pub fn draw(&mut self, panes: &[Pane]) -> anyhow::Result<()> {
        let height = terminal::size()?.1;

        let viewable_panes = panes
            .iter()
            .filter(|pane| !pane.is_empty())
            .collect::<Vec<&Pane>>();

        if height < viewable_panes.len() as u16 {
            return crossterm::execute!(
                self.writer,
                terminal::Clear(terminal::ClearType::FromCursorDown),
                style::Print("⚠️ Insufficient Space"),
            )
            .map_err(anyhow::Error::from);
        }

        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.position.0, self.position.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        let mut used = 0;

        let mut current_cursor_y = height.saturating_sub(self.position.1);
        for (i, pane) in viewable_panes.iter().enumerate() {
            let rows = pane.extract(
                1.max(
                    (height as usize)
                        // -1 in this context signifies the exclusion of the current pane.
                        .saturating_sub(used + viewable_panes.len() - 1 - i),
                ),
            );
            used += rows.len();
            for (j, row) in rows.iter().enumerate() {
                crossterm::queue!(self.writer, style::Print(row.styled_display()))?;

                current_cursor_y = current_cursor_y.saturating_sub(1);

                if ((i == viewable_panes.len() - 1 && j != rows.len() - 1)
                    || i != viewable_panes.len() - 1)
                    && current_cursor_y == 0
                {
                    crossterm::queue!(self.writer, terminal::ScrollUp(1))?;
                    self.position.1 = self.position.1.saturating_sub(1);
                }

                crossterm::queue!(self.writer, cursor::MoveToNextLine(1))?;
            }
        }
        self.writer.flush()?;
        Ok(())
    }
}
