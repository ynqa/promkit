use std::io::Write;

use anyhow::{ensure, Result};

use crate::{engine::Engine, pane::Pane};

// Session
pub struct Terminal {
    offset: (u16, u16),
}

impl Terminal {
    pub fn start_session<W: Write>(
        engine: &mut Engine<W>,
        guaranteed_height: usize,
    ) -> Result<Self> {
        let mut offset = engine.position()?;

        if (offset.1 as usize) < guaranteed_height {
            engine.clear()?;
            offset = engine.position()?;
            ensure!(
                (offset.1 as usize) < guaranteed_height,
                "Terminal window does not have enough vertical space to render UI."
            );
        }

        Ok(Self { offset })
    }

    pub fn draw<W: Write>(&mut self, engine: &mut Engine<W>, panes: Vec<Pane>) -> Result<()> {
        engine.move_to(self.offset)?;
        let mut start_height = self.offset.1 as usize;
        let terminal_size = engine.size()?;

        for pane in panes {
            // When the cursor is at top of terminal,
            // we must consider the last scrolling up.
            let rows = pane
                .extract(terminal_size.1 as usize - vec![start_height, 1].iter().max().unwrap());
            for row in &rows {
                engine.write(row)?;
            }

            if engine.is_bottom()? {
                engine.scroll_up(1)?;
                self.offset.1 = self.offset.1.saturating_sub(1);
                start_height = start_height.saturating_sub(1);
            }
            start_height += &rows.len();
        }
        Ok(())
    }
}
