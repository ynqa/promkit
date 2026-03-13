use promkit_core::{Widget, grapheme::StyledGraphemes};

#[path = "text/text.rs"]
mod inner;
pub use inner::Text;
pub mod config;
pub use config::Config;

/// Represents the state of a text-based component within the application.
///
/// This state encapsulates the properties and
/// behaviors specific to text handling,
#[derive(Clone, Default)]
pub struct State {
    /// The text to be rendered.
    pub text: Text,
    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl State {
    pub fn replace(&mut self, state: Self) {
        *self = state;
    }

    pub fn replace_text(&mut self, text: Vec<StyledGraphemes>) {
        self.text.replace_contents(text);
    }
}

impl Widget for State {
    fn create_graphemes(&self, _width: u16, height: u16) -> StyledGraphemes {
        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let lines = self
            .text
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.text.position() && *i < self.text.position() + height)
            .map(|(_, item)| {
                if let Some(style) = &self.config.style {
                    item.clone().apply_style(*style)
                } else {
                    item.clone()
                }
            });

        StyledGraphemes::from_lines(lines)
    }
}
