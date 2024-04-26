use crate::{crossterm::style::ContentStyle, grapheme::StyledGraphemes, pane::Pane, PaneFactory};

/// Represents the state of a text-based component within the application.
///
/// This state encapsulates the properties and
/// behaviors specific to text handling,
#[derive(Clone)]
pub struct State {
    /// The text to be rendered.
    pub text: String,

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
        let (matrix, _) = StyledGraphemes::from_str(&self.text, self.style).matrixify(
            width as usize,
            height as usize,
            0,
        );
        Pane::new(matrix, 0)
    }
}
