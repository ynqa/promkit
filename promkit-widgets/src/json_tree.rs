use promkit_core::{Widget, grapheme::StyledGraphemes};

#[path = "json_tree/json_tree.rs"]
mod inner;
pub use inner::JsonTree;
pub mod config;
pub use config::Config;
pub mod jsonz;

/// Represents the state of a JSON tree within the application.
///
/// This struct holds the current JSON tree and provides
/// methods to navigate and manipulate the tree according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    /// The current JSON tree being displayed.
    pub tree: JsonTree,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self, width: u16, height: u16) -> StyledGraphemes {
        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.tree.extract_rows_from_current(height);
        let formatted_rows = self.config.format_for_terminal_display(&rows, width);

        StyledGraphemes::from_lines(formatted_rows)
    }
}
