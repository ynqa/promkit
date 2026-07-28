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
    anchor: (u16, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DrawPlan {
    row_count: usize,
    clear_position: (u16, u16),
    draw_position: (u16, u16),
    scroll_down: u16,
    visible_height: usize,
    resulting_position: (u16, u16),
}

impl DrawPlan {
    fn try_new(
        row_count: usize,
        terminal_height: u16,
        anchor: (u16, u16),
        position: (u16, u16),
    ) -> anyhow::Result<Self> {
        if row_count > terminal_height as usize {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
        }

        let last_terminal_row = terminal_height.saturating_sub(1);
        let anchor_row = anchor.1.min(last_terminal_row);
        let position_row = position.1.min(last_terminal_row);
        let target_row = if row_count == 0 {
            anchor_row
        } else {
            let row_count =
                u16::try_from(row_count).expect("row count is bounded by terminal height");
            anchor_row.min(terminal_height.saturating_sub(row_count))
        };
        let scroll_down = target_row.saturating_sub(position_row);
        let draw_row = position_row.saturating_add(scroll_down);

        Ok(Self {
            row_count,
            clear_position: (position.0, draw_row),
            draw_position: (anchor.0, draw_row),
            scroll_down,
            visible_height: terminal_height.saturating_sub(draw_row) as usize,
            resulting_position: (anchor.0, target_row),
        })
    }

    fn scroll_up_after(self, row_index: usize) -> bool {
        let completed_rows = row_index.saturating_add(1);
        completed_rows < self.row_count && completed_rows >= self.visible_height.max(1)
    }
}

impl Terminal {
    pub fn new(position: (u16, u16)) -> Self {
        Self {
            position,
            anchor: position,
        }
    }

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
        self.draw_rows_at_height(panes, height)
    }

    pub(crate) fn draw_rows_at_height<R>(
        &mut self,
        panes: &[Vec<R>],
        height: u16,
    ) -> anyhow::Result<()>
    where
        R: Borrow<StyledGraphemes>,
    {
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
        let row_count = panes.iter().map(Vec::len).sum::<usize>();
        let plan = DrawPlan::try_new(row_count, height, self.anchor, self.position)?;

        crossterm::queue!(
            writer,
            terminal::BeginSynchronizedUpdate,
            terminal::DisableLineWrap,
        )?;
        if plan.scroll_down > 0 {
            crossterm::queue!(writer, terminal::ScrollDown(plan.scroll_down))?;
        }
        crossterm::queue!(
            writer,
            cursor::MoveTo(plan.clear_position.0, plan.clear_position.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
            cursor::MoveTo(plan.draw_position.0, plan.draw_position.1),
        )?;

        for (row_index, row) in panes.iter().flatten().enumerate() {
            crossterm::queue!(writer, style::Print(row.borrow().styled_display()))?;

            if plan.scroll_up_after(row_index) {
                crossterm::queue!(writer, terminal::ScrollUp(1))?;
            }

            crossterm::queue!(writer, cursor::MoveToNextLine(1))?;
        }
        crossterm::queue!(
            writer,
            terminal::EnableLineWrap,
            terminal::EndSynchronizedUpdate
        )?;
        writer.flush()?;
        self.position = plan.resulting_position;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod terminal {
        use super::*;

        mod draw_rows_to {
            use super::*;
            use crate::crossterm::terminal as crossterm_terminal;

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
                let mut terminal = Terminal::new((0, 7));
                let mut output = Vec::new();

                terminal.draw_rows_to(&mut output, &rows(8), 10).unwrap();

                let clear = command_offset(
                    &output,
                    crossterm_terminal::Clear(crossterm_terminal::ClearType::FromCursorDown),
                );
                let scroll = command_offset(&output, crossterm_terminal::ScrollUp(1));

                assert!(clear < scroll);
                assert_eq!(terminal.position, (0, 2));
            }

            #[test]
            fn shrinking_frame_scrolls_preceding_output_back_down_before_redrawing() {
                let mut terminal = Terminal::new((0, 7));
                terminal
                    .draw_rows_to(&mut Vec::new(), &rows(8), 10)
                    .unwrap();
                assert_eq!(terminal.position, (0, 2));

                let mut output = Vec::new();
                terminal.draw_rows_to(&mut output, &rows(3), 10).unwrap();

                let scroll_down = command_offset(&output, crossterm_terminal::ScrollDown(5));
                let clear = command_offset(
                    &output,
                    crossterm_terminal::Clear(crossterm_terminal::ClearType::FromCursorDown),
                );
                let draw = command_bytes(cursor::MoveTo(0, 7));
                let draw = output
                    .windows(draw.len())
                    .enumerate()
                    .filter(|(_, window)| *window == draw)
                    .map(|(offset, _)| offset)
                    .nth(1)
                    .expect("expected a move from the clear position to the draw position");

                assert!(scroll_down < clear);
                assert!(clear < draw);
                assert_eq!(terminal.position, (0, 7));
            }

            #[test]
            fn draw_frame_controls_wrapping_inside_a_synchronized_update() {
                let mut terminal = Terminal::new((0, 0));
                let mut output = Vec::new();

                terminal.draw_rows_to(&mut output, &rows(1), 10).unwrap();

                let begin = command_offset(&output, crossterm_terminal::BeginSynchronizedUpdate);
                let disable_wrap = command_offset(&output, crossterm_terminal::DisableLineWrap);
                let enable_wrap = command_offset(&output, crossterm_terminal::EnableLineWrap);
                let end = command_offset(&output, crossterm_terminal::EndSynchronizedUpdate);

                assert!(begin < disable_wrap);
                assert!(disable_wrap < enable_wrap);
                assert!(enable_wrap < end);
            }

            #[test]
            fn trailing_empty_panes_do_not_trigger_scrolling() {
                let mut terminal = Terminal::new((0, 9));
                let panes: Vec<Vec<StyledGraphemes>> =
                    vec![vec![StyledGraphemes::from("only row")], Vec::new()];

                terminal.draw_rows_to(&mut Vec::new(), &panes, 10).unwrap();

                assert_eq!(terminal.position, (0, 9));
            }
        }
    }
}
