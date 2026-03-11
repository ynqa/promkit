use promkit_core::{GraphemeFactory, grapheme::StyledGraphemes};

pub mod node;
use node::Kind;
#[path = "tree/tree.rs"]
mod inner;
pub use inner::Tree;
pub mod config;
pub use config::Config;

/// Represents the state of a tree structure within the application.
///
/// This state includes not only the tree itself but also various properties
/// that affect how the tree is displayed and interacted with. These properties
/// include symbols for folded and unfolded items, styles for active and inactive
/// items, the number of lines available for rendering, and the indentation level
/// for child items in the tree.
#[derive(Clone)]
pub struct State {
    pub tree: Tree,
    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl GraphemeFactory for State {
    fn create_graphemes(&self, _width: u16, height: u16) -> StyledGraphemes {
        let symbol = |kind: &Kind| -> &str {
            match kind {
                Kind::Folded { .. } => &self.config.folded_symbol,
                Kind::Unfolded { .. } => &self.config.unfolded_symbol,
            }
        };

        let indent = |kind: &Kind| -> usize {
            match kind {
                Kind::Folded { path, .. } | Kind::Unfolded { path, .. } => {
                    path.len() * self.config.indent
                }
            }
        };

        let id = |kind: &Kind| -> String {
            match kind {
                Kind::Folded { id, .. } | Kind::Unfolded { id, .. } => id.clone(),
            }
        };

        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let kinds = self.tree.kinds();
        let lines = kinds
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.tree.position() && *i < self.tree.position() + height)
            .map(|(i, kind)| {
                if i == self.tree.position() {
                    StyledGraphemes::from_str(
                        format!("{}{}{}", symbol(kind), " ".repeat(indent(kind)), id(kind),),
                        self.config.active_item_style,
                    )
                } else {
                    StyledGraphemes::from_str(
                        format!(
                            "{}{}{}",
                            " ".repeat(StyledGraphemes::from(symbol(kind)).widths()),
                            " ".repeat(indent(kind)),
                            id(kind),
                        ),
                        self.config.inactive_item_style,
                    )
                }
            });

        StyledGraphemes::from_lines(lines)
    }
}
