use crate::{crossterm::style::ContentStyle, grapheme::StyledGraphemes, pane::Pane, PaneFactory};

/// Represents the state of a text-based component within the application.
///
/// This state encapsulates the properties and
/// behaviors specific to text handling,
#[derive(Clone)]
pub struct State {
    /// The text to be rendered.
    pub text: String,
    /// The index of the matrix to start rendering from.
    pub matrix_index: usize,

    /// Style for the text string.
    pub style: ContentStyle,
}

impl State {
    pub fn replace(&mut self, renderer: Self) {
        *self = renderer;
    }
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let (matrix, offset) = StyledGraphemes::from_str(&self.text, self.style).matrixify(
            width as usize,
            height as usize,
            self.matrix_index,
        );
        Pane::new(matrix, offset)
    }
}

impl State {
    pub fn down(&mut self) {
        self.matrix_index += 1;
    }

    pub fn up(&mut self) {
        if self.matrix_index > 0 {
            self.matrix_index -= 1;
        }
    }
}
