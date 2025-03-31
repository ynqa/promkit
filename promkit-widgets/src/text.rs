use promkit_core::{Pane, PaneFactory, crossterm::style::ContentStyle};

mod text;
pub use text::Text;

/// Represents the state of a text-based component within the application.
///
/// This state encapsulates the properties and
/// behaviors specific to text handling,
#[derive(Clone, Default)]
pub struct State {
    /// The text to be rendered.
    pub text: Text,

    /// Style for the text string.
    pub style: ContentStyle,

    /// Maximum number of lines to display.
    pub lines: Option<usize>,
}

impl State {
    pub fn replace(&mut self, renderer: Self) {
        *self = renderer;
    }
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let matrix = self
            .text
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.text.position() && *i < self.text.position() + height)
            .map(|(_, item)| item.clone().apply_style(self.style))
            .fold((vec![], 0), |(mut acc, pos), item| {
                let rows = item.matrixify(width as usize, height, 0).0;
                if pos < self.text.position() + height {
                    acc.extend(rows);
                }
                (acc, pos + 1)
            });

        Pane::new(matrix.0, 0)
    }
}
