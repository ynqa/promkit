use std::io::Write;

use anyhow::Result;

use crate::{
    crossterm::{cursor, terminal},
    engine::Engine,
    grapheme::Graphemes,
};

// Session
pub struct Terminal {}

impl Terminal {
    pub fn start_session() -> Result<Self> {
        let _size = terminal::size()?;
        let _start_position = cursor::position()?;

        // let required_height = panes.iter().fold(0, |mut acc, pane| {
        //     acc += pane.requirement.guaranteed_height;
        //     acc
        // });

        // ensure!(
        //     size.1 >= required_height,
        //     "Terminal window does not have enough vertical space to render UI."
        // );

        Ok(Self {})
    }

    pub fn render(&mut self, _layout: &Vec<Graphemes>) -> Result<()> {
        // TBD.
        // self.panes[0].render(contents);
        // call `draw`.
        Ok(())
    }

    pub fn draw<W: Write>(&mut self, _engine: &mut Engine<W>) -> Result<()> {
        // for pane in self.panes.iter() {
        //     for row in pane.rows.iter() {
        //         engine.write(row)?;
        //     }
        // }
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
