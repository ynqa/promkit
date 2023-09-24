use std::{
    fmt::Write,
    ops::{Deref, DerefMut},
};

use crate::{grapheme::Graphemes, Result};

pub struct Requirement {
    /// Determina the order which the panes are drawn.
    pub priority_to_draw: u16,
    /// Determine the order which the panes are assigned vertical space
    /// when the available space is limited.
    pub priority_to_occupy_height: u16,
    /// Minimum amount of vertical space that the pane must occupy,
    /// even if the screen is not large enough to accommodate it fully.
    pub guaranteed_height: u16,
}

pub struct Pane {
    pub requirement: Requirement,
}

impl Pane {
    pub fn new(requirement: Requirement) -> Self {
        Self { requirement }
    }

    pub fn render(&mut self, _viewport: (u16, u16), _layout: &Vec<Graphemes>) -> Result<u16> {
        // // Merge all contents into one object.
        // let obj = contents
        //     .iter()
        //     .fold(Graphemes::default(), |mut acc, content| {
        //         acc.extend_from_slice(&content);
        //         acc
        //     });

        // // Convert to my layout.
        // self.rows = obj.matrixify(viewport.0 as usize);
        // Ok(self.rows.len() as u16)
        Ok(0)
    }

    fn draw<W: Write>(&mut self, _start_position: (u16, u16), _size: (u16, u16)) -> Result<()> {
        Ok(())
    }
}

pub struct Panes(pub Vec<Pane>);

impl Deref for Panes {
    type Target = Vec<Pane>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Panes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
