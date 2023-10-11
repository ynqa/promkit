use std::io::Write;

use anyhow::{ensure, Result};

use crate::{engine::Engine, pane::Pane};

// Session
pub struct Terminal {
    position: (u16, u16),
}

impl Terminal {
    pub fn start_session<W: Write>(engine: &mut Engine<W>) -> Result<Self> {
        Ok(Self {
            position: engine.position()?,
        })
    }

    pub fn draw<W: Write>(&mut self, engine: &mut Engine<W>, panes: Vec<Pane>) -> Result<()> {
        let terminal_height = engine.size()?.1 as usize;
        ensure!(
            terminal_height > panes.len(),
            "Terminal window does not have enough vertical space to render UI."
        );

        engine.move_to(self.position)?;
        engine.clear_from_cursor_down()?;

        let mut used = 0;

        for (i, pane) in panes.iter().enumerate() {
            let rows = pane.extract(
                1.max(
                    terminal_height
                        // -1 in this context signifies the exclusion of the current pane.
                        // .saturating_sub(used + panes.len() - 1 - i),
                        .saturating_sub(used + panes.len() - 1 - i),
                ),
            );
            used += rows.len();
            for row in &rows {
                engine.write(row.styled_display())?;

                // Note that the last line is not utilized.
                // The cursor is positioned at the zero point on the last line
                // after writing a row when the cursor is at the bottom.
                if engine.is_bottom()? && self.position.1 != 0 {
                    engine.scroll_up(1)?;
                    self.position.1 -= 1;
                }
                engine.move_to_next_line()?;
            }
        }
        Ok(())
    }
}
