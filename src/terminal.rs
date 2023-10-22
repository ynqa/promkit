use std::io::Write;

use crate::{engine::Engine, error::Result, pane::Pane};

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

    pub fn draw<W: Write>(&mut self, engine: &mut Engine<W>, panes: Vec<Pane>) -> Result {
        let terminal_height = engine.size()?.1 as usize;
        let viewable_panes = panes
            .iter()
            .filter(|pane| !pane.is_empty())
            .collect::<Vec<&Pane>>();

        if terminal_height < viewable_panes.len() {
            return engine
                .write("Terminal window does not have enough vertical space to render UI.");
        }

        engine.move_to(self.position)?;
        engine.clear_from_cursor_down()?;

        let mut used = 0;

        for (i, pane) in viewable_panes.iter().enumerate() {
            let rows = pane.extract(
                1.max(
                    terminal_height
                        // -1 in this context signifies the exclusion of the current pane.
                        .saturating_sub(used + viewable_panes.len() - 1 - i),
                ),
            );
            used += rows.len();
            for row in &rows {
                engine.write(row.styled_display())?;

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
