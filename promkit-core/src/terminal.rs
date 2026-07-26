use std::{
    borrow::Borrow,
    io::{self, Write},
};

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
    pub fn draw_rows<R>(&mut self, panes: &[Vec<R>]) -> anyhow::Result<()>
    where
        R: Borrow<StyledGraphemes>,
    {
        let (_, height) = terminal::size()?;
        let mut stdout = io::stdout();
        self.draw_rows_to(&mut stdout, panes, height)
    }

    fn draw_rows_to<W, R>(
        &mut self,
        writer: &mut W,
        panes: &[Vec<R>],
        height: u16,
    ) -> anyhow::Result<()>
    where
        W: Write,
        R: Borrow<StyledGraphemes>,
    {
        let visible_height = height.saturating_sub(self.position.1);
        let row_count = panes.iter().map(Vec::len).sum::<usize>();

        if row_count > height as usize {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
        }

        crossterm::queue!(
            writer,
            cursor::MoveTo(self.position.0, self.position.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        let mut remaining_lines = visible_height;

        for (pane_index, rows) in panes.iter().enumerate() {
            for (row_index, row) in rows.iter().enumerate() {
                crossterm::queue!(writer, style::Print(row.borrow().styled_display()))?;

                remaining_lines = remaining_lines.saturating_sub(1);

                // Determine if scrolling is needed:
                // - We need to scroll if we've reached the bottom of the terminal (remaining_lines == 0)
                // - AND we have more content to display (either more rows in current pane or more panes)
                let is_last_pane = pane_index == panes.len() - 1;
                let is_last_row_in_pane = row_index == rows.len() - 1;
                let has_more_content = !(is_last_pane && is_last_row_in_pane);

                if has_more_content && remaining_lines == 0 {
                    crossterm::queue!(writer, terminal::ScrollUp(1))?;
                    self.position.1 = self.position.1.saturating_sub(1);
                }

                crossterm::queue!(writer, cursor::MoveToNextLine(1))?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(count: usize) -> Vec<Vec<StyledGraphemes>> {
        vec![
            (0..count)
                .map(|index| StyledGraphemes::from(format!("row {index}")))
                .collect(),
        ]
    }

    fn command_bytes(command: impl crate::crossterm::Command) -> Vec<u8> {
        let mut output = Vec::new();
        crossterm::queue!(output, command).unwrap();
        output
    }

    fn command_offset(output: &[u8], command: impl crate::crossterm::Command) -> usize {
        let command = command_bytes(command);
        output
            .windows(command.len())
            .position(|window| window == command)
            .expect("expected terminal command was not emitted")
    }

    #[test]
    fn growing_frame_scrolls_only_after_clearing_the_previous_frame() {
        let mut terminal = Terminal { position: (0, 7) };
        let mut output = Vec::new();

        terminal.draw_rows_to(&mut output, &rows(8), 10).unwrap();

        let clear = command_offset(
            &output,
            terminal::Clear(terminal::ClearType::FromCursorDown),
        );
        let scroll = command_offset(&output, terminal::ScrollUp(1));

        assert!(clear < scroll);
        assert_eq!(terminal.position, (0, 2));
    }

    #[test]
    fn shrinking_frame_scrolls_preceding_output_back_down_before_redrawing() {
        let mut terminal = Terminal { position: (0, 7) };
        terminal
            .draw_rows_to(&mut Vec::new(), &rows(8), 10)
            .unwrap();
        assert_eq!(terminal.position, (0, 2));

        let mut output = Vec::new();
        terminal.draw_rows_to(&mut output, &rows(3), 10).unwrap();

        let scroll_down = command_offset(&output, terminal::ScrollDown(5));
        let clear = command_offset(
            &output,
            terminal::Clear(terminal::ClearType::FromCursorDown),
        );
        let draw = command_offset(&output, cursor::MoveTo(0, 7));

        assert!(scroll_down < clear);
        assert!(clear < draw);
        assert_eq!(terminal.position, (0, 7));
    }

    #[test]
    fn draw_frame_controls_wrapping_inside_a_synchronized_update() {
        let mut terminal = Terminal { position: (0, 0) };
        let mut output = Vec::new();

        terminal.draw_rows_to(&mut output, &rows(1), 10).unwrap();

        let begin = command_offset(&output, terminal::BeginSynchronizedUpdate);
        let disable_wrap = command_offset(&output, terminal::DisableLineWrap);
        let enable_wrap = command_offset(&output, terminal::EnableLineWrap);
        let end = command_offset(&output, terminal::EndSynchronizedUpdate);

        assert!(begin < disable_wrap);
        assert!(disable_wrap < enable_wrap);
        assert!(enable_wrap < end);
    }

    #[test]
    fn trailing_empty_panes_do_not_trigger_scrolling() {
        let mut terminal = Terminal { position: (0, 9) };
        let panes: Vec<Vec<StyledGraphemes>> =
            vec![vec![StyledGraphemes::from("only row")], Vec::new()];

        terminal.draw_rows_to(&mut Vec::new(), &panes, 10).unwrap();

        assert_eq!(terminal.position, (0, 9));
    }
}
