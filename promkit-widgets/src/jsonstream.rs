use promkit_core::{Widget, grapheme::StyledGraphemes};

#[path = "jsonstream/jsonstream.rs"]
mod inner;
pub use inner::JsonStream;
pub mod config;
pub use config::Config;
pub mod jsonz;

/// Represents the state of a JSON stream within the application.
///
/// This struct holds the current JSON stream being processed and provides
/// methods to interact with and manipulate the stream according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    /// The current JSON stream being processed.
    pub stream: JsonStream,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self, width: u16, height: u16) -> StyledGraphemes {
        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.stream.extract_rows_from_current(height);
        let formatted_rows = self.config.format_for_terminal_display(&rows, width);

        StyledGraphemes::from_lines(formatted_rows)
    }
}
