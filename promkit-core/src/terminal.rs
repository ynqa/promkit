use std::io::{self, Write};

use crate::{
    crossterm::{cursor, style, terminal},
    grapheme::StyledGraphemes,
};

pub struct Terminal {
    /// The current cursor position within the terminal.
    pub position: (u16, u16),
}

impl Terminal {
    /// Draws content that still needs terminal-width wrapping.
    pub fn draw(&mut self, graphemes: &[StyledGraphemes]) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let viewable_rows = graphemes
            .iter()
            .map(|graphemes| graphemes.wrapped_lines(width as usize))
            .filter(|rows| !rows.is_empty())
            .collect::<Vec<Vec<StyledGraphemes>>>();

        if height < viewable_rows.len() as u16 {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
        }

        let mut used = 0;
        let mut panes = Vec::with_capacity(viewable_rows.len());
        for (pane_index, rows) in viewable_rows.iter().enumerate() {
            let max_rows = 1
                .max((height as usize).saturating_sub(used + viewable_rows.len() - 1 - pane_index));
            let rows = rows.iter().take(max_rows).cloned().collect::<Vec<_>>();
            used += rows.len();
            panes.push(rows);
        }

        self.draw_rows(&panes)
    }

    /// Draws panes that have already been wrapped and clipped by the renderer.
    pub fn draw_rows(&mut self, panes: &[Vec<StyledGraphemes>]) -> anyhow::Result<()> {
        let (_, height) = terminal::size()?;
        let visible_height = height.saturating_sub(self.position.1);
        let row_count = panes.iter().map(Vec::len).sum::<usize>();

        if row_count > height as usize {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
        }

        crossterm::queue!(
            io::stdout(),
            cursor::MoveTo(self.position.0, self.position.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        let mut remaining_lines = visible_height;

        for (pane_index, rows) in panes.iter().enumerate() {
            for (row_index, row) in rows.iter().enumerate() {
                crossterm::queue!(io::stdout(), style::Print(row.styled_display()))?;

                remaining_lines = remaining_lines.saturating_sub(1);

                // Determine if scrolling is needed:
                // - We need to scroll if we've reached the bottom of the terminal (remaining_lines == 0)
                // - AND we have more content to display (either more rows in current pane or more panes)
                let is_last_pane = pane_index == panes.len() - 1;
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
