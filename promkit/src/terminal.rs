use std::io::{self, Write};

use crate::{
    crossterm::{cursor, style, terminal},
    pane::Pane,
};

pub struct Terminal {
    /// The current cursor position within the terminal.
    pub position: (u16, u16),
}

impl Terminal {
    pub fn start_session(panes: &[Pane]) -> anyhow::Result<Self> {
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
                crossterm::queue!(io::stdout(), terminal::ScrollUp(1))?;
            }
            crossterm::queue!(io::stdout(), cursor::MoveToNextLine(1))?;
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
                io::stdout(),
                terminal::ScrollUp(lines as u16),
                cursor::MoveToPreviousLine(lines as u16),
            )?;
        }

        io::stdout().flush()?;

        Ok(Self {
            position: cursor::position()?,
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
                io::stdout(),
                terminal::Clear(terminal::ClearType::FromCursorDown),
                style::Print("⚠️ Insufficient Space"),
            )
            .map_err(anyhow::Error::from);
        }

        crossterm::queue!(
            io::stdout(),
            cursor::MoveTo(self.position.0, self.position.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        let mut used = 0;

        let mut remaining_lines = height.saturating_sub(self.position.1);

        for (pane_index, pane) in viewable_panes.iter().enumerate() {
            // We need to ensure each pane gets at least 1 row
            let max_rows = 1.max(
                (height as usize).saturating_sub(used + viewable_panes.len() - 1 - pane_index),
            );

            let rows = pane.extract(max_rows);
            used += rows.len();

            for (row_index, row) in rows.iter().enumerate() {
                crossterm::queue!(io::stdout(), style::Print(row.styled_display()))?;

                remaining_lines = remaining_lines.saturating_sub(1);

                // Determine if scrolling is needed:
                // - We need to scroll if we've reached the bottom of the terminal (remaining_lines == 0)
                // - AND we have more content to display (either more rows in current pane or more panes)
                let is_last_pane = pane_index == viewable_panes.len() - 1;
                let is_last_row_in_pane = row_index == rows.len() - 1;
                let has_more_content = !(is_last_pane && is_last_row_in_pane);

                if has_more_content && remaining_lines == 0 {
                    crossterm::queue!(io::stdout(), terminal::ScrollUp(1))?;
                    self.position.1 = self.position.1.saturating_sub(1);
                }

                crossterm::queue!(io::stdout(), cursor::MoveToNextLine(1))?;
            }
        }
        io::stdout().flush()?;
        Ok(())
    }
}
