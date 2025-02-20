use crate::{core::cursor::Cursor, grapheme::StyledGraphemes};

mod state;
pub use state::State;

#[derive(Clone)]
pub struct Text(Cursor<Vec<StyledGraphemes>>);

impl Default for Text {
    fn default() -> Self {
        Self(Cursor::new(vec![], 0, false))
    }
}

impl<T: AsRef<str>> From<T> for Text {
    fn from(text: T) -> Self {
        let lines: Vec<StyledGraphemes> = text
            .as_ref()
            .split('\n')
            .map(StyledGraphemes::from)
            .collect();
        Self(Cursor::new(lines, 0, false))
    }
}

impl Text {
    /// Returns a reference to the vector of items in the listbox.
    pub fn items(&self) -> &Vec<StyledGraphemes> {
        self.0.contents()
    }

    /// Returns the current position of the cursor within the listbox.
    pub fn position(&self) -> usize {
        self.0.position()
    }

    /// Moves the cursor backward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.0.backward()
    }

    /// Moves the cursor forward in the listbox, if possible.
    /// Returns `true` if the cursor was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.0.forward()
    }
}
