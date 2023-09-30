use std::io::Write;

use anyhow::Result;

use crate::{engine::Engine, pane::Pane};

// Session
pub struct Terminal {
    top_pane_start_position: (u16, u16),
}

impl Terminal {
    pub fn start_session<W: Write>(engine: &mut Engine<W>) -> Result<Self> {
        let top_pane_start_position = engine.position()?;

        // let required_height = panes.iter().fold(0, |mut acc, pane| {
        //     acc += pane.requirement.guaranteed_height;
        //     acc
        // });

        // ensure!(
        //     size.1 >= required_height,
        //     "Terminal window does not have enough vertical space to render UI."
        // );

        Ok(Self {
            top_pane_start_position,
        })
    }

    pub fn draw<W: Write>(&mut self, engine: &mut Engine<W>, panes: Vec<Pane>) -> Result<()> {
        engine.move_to(self.top_pane_start_position)?;
        let mut start_height = self.top_pane_start_position.1 as usize;
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
                self.top_pane_start_position.1 = self.top_pane_start_position.1.saturating_sub(1);
                start_height = start_height.saturating_sub(1);
            }
            start_height += &rows.len();
        }
        Ok(())
    }

    /// Update the internal state to match the new screen size,
    /// allowing it to correctly render content within the new bounds.
    /// It conducts the screen clear also.
    pub fn reshape<W: Write>(&mut self, engine: &mut Engine<W>) -> Result<()> {
        engine.clear()?;
        Ok(())
    }
}
