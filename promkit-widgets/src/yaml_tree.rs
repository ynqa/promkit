use promkit_core::{Widget, grapheme::StyledGraphemes};

#[path = "yaml_tree/yaml_tree.rs"]
mod inner;
pub use inner::YamlTree;
pub mod config;
pub use config::Config;
pub mod yamlz;

/// Represents the state of a YAML tree within the application.
#[derive(Clone)]
pub struct State {
    /// The current YAML tree being displayed.
    pub tree: YamlTree,

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
