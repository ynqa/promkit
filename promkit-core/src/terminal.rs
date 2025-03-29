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
    pub fn draw(&mut self, panes: &[Pane]) -> anyhow::Result<()> {
        let height = terminal::size()?.1;

        let viewable_panes = panes
            .iter()
            .filter(|pane| !pane.is_empty())
            .collect::<Vec<&Pane>>();

        if height < viewable_panes.len() as u16 {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
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
