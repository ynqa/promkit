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
        // Ensure that there is at least one additional line of space available
        // below the last pane for UI rendering purposes.
        ensure!(
            (engine.size()?.1 as usize) >= panes.len() + 1,
            "Terminal window does not have enough vertical space to render UI."
        );

        engine.move_to(self.position)?;
        engine.clear_from_cursor_down()?;

        let mut offset_per_pane = self.position.1 as usize;
        let terminal_size = engine.size()?;

        for pane in &panes {
            let rows = pane.extract(terminal_size.1 as usize - offset_per_pane);
            for row in &rows {
                engine.write(row)?;

                // Note that the last line is not utilized.
                // The cursor is positioned at the zero point on the last line
                // after writing a row when the cursor is at the bottom.
                if engine.is_bottom()? {
                    engine.scroll_up(1)?;
                    self.position.1 = self.position.1.saturating_sub(1);
                    offset_per_pane = offset_per_pane.saturating_sub(1);
                }
            }

            offset_per_pane += &rows.len();
            engine.move_to_next_line()?;
        }
        Ok(())
    }
}
