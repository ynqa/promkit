use std::io::Write;

use anyhow::Result;

use crate::{engine::Engine, pane::Pane};

// Session
pub struct Terminal {
    start_position: (u16, u16),
}

impl Terminal {
    pub fn start_session<W: Write>(engine: &mut Engine<W>) -> Result<Self> {
        let start_position = engine.position()?;

        // let required_height = panes.iter().fold(0, |mut acc, pane| {
        //     acc += pane.requirement.guaranteed_height;
        //     acc
        // });

        // ensure!(
        //     size.1 >= required_height,
        //     "Terminal window does not have enough vertical space to render UI."
        // );

        Ok(Self { start_position })
    }

    pub fn draw<W: Write>(&mut self, engine: &mut Engine<W>, panes: Vec<Pane>) -> Result<()> {
        engine.move_to(self.start_position)?;
        let mut start_height = self.start_position.1 as usize;
        let terminal_height = engine.size()?.1;

        for pane in panes {
            let rows = pane.extract(terminal_height as usize - start_height);
            for row in &rows {
                engine.write(row)?;
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
