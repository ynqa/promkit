use promkit_core::{Pane, PaneFactory};

#[path = "jsonstream/jsonstream.rs"]
mod inner;
pub use inner::JsonStream;
pub mod format;
use format::Formatter;
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

    /// Theme configuration for styling the JSON output.
    pub formatter: Formatter,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.stream.extract_rows_from_current(height);
        let formatted_rows = self.formatter.format_for_terminal_display(&rows, width);

        Pane::new(formatted_rows, 0)
    }
}
