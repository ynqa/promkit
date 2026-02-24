use promkit_core::{Pane, PaneFactory, grapheme::StyledGraphemes};

pub mod node;
use node::Kind;
#[path = "tree/tree.rs"]
mod inner;
pub use inner::Tree;
pub mod format;
pub use format::{Config, Formatter};

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

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
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

        let matrix = self
            .tree
            .kinds()
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
            })
            .fold((vec![], 0), |(mut acc, pos), item| {
                let rows = item.matrixify(width as usize, height, 0).0;
                if pos < self.tree.position() + height {
                    acc.extend(rows);
                }
                (acc, pos + 1)
            });

        Pane::new(matrix.0, 0)
    }
}
