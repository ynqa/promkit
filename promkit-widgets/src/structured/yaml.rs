use promkit_core::{Widget, grapheme::StyledGraphemes};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
pub mod yamlz;

/// Represents YAML view state within the application.
#[derive(Clone)]
pub struct State {
    /// The current YAML document being displayed.
    pub document: Document,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self, width: u16, height: u16) -> StyledGraphemes {
        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.document.extract_rows_from_current(height);
        let formatted_rows = self.config.render_terminal_rows(&rows, width);

        StyledGraphemes::from_lines(formatted_rows)
    }
}
