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

        let mut used = 0;

        for pane in &panes {
            let rows = pane.extract(engine.size()?.1 as usize - used);
            used += rows.len();
            for row in &rows {
                engine.write(row)?;

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
