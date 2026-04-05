use promkit_core::{Widget, grapheme::StyledGraphemes};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
pub mod jsonz;

use crate::structured::PrettyRender;

/// Represents JSON view state within the application.
///
/// This struct holds the current JSON document and provides
/// methods to navigate and manipulate rows according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    /// The current JSON document being displayed.
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

impl State {
    /// Formats the raw JSON data into a pretty-printed string with indentation.
    pub fn render_pretty_json(&self) -> String {
        self.document.rows().render_pretty(self.config.indent)
    }
}
