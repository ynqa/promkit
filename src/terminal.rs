use std::io::Write;

use crate::{engine::Engine, error::Result, pane::Pane};

/// Represents a terminal session,
/// managing the display of panes within the terminal window.
pub struct Terminal {
    /// The current cursor position within the terminal.
    position: (u16, u16),
}

impl Terminal {
    /// Starts a new terminal session, initializing the cursor position.
    ///
    /// # Arguments
    ///
    /// * `engine` - A mutable reference to the Engine,
    /// which manages terminal operations.
    ///
    /// # Returns
    ///
    /// A result containing the new Terminal instance or an error.
    pub fn start_session<W: Write>(engine: &mut Engine<W>) -> Result<Self> {
        Ok(Self {
            position: engine.position()?,
        })
    }

    /// Draws the provided panes within the terminal window.
    ///
    /// # Arguments
    ///
    /// * `engine` - A mutable reference to the Engine,
    /// which manages terminal operations.
    /// * `panes` - A vector of Pane instances to be displayed.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal window
    /// does not have enough vertical space to render the UI.
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
