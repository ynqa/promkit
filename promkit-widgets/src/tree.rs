use promkit_core::{
    PaneFactory, crossterm::style::ContentStyle, grapheme::StyledGraphemes, pane::Pane,
};

pub mod node;
use node::Kind;
mod tree;
pub use tree::Tree;

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

    /// Symbol representing folded items.
    pub folded_symbol: String,
    /// Symbol representing unfolded items.
    pub unfolded_symbol: String,

    /// Style for the selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected lines.
    pub inactive_item_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,

    /// The number of spaces used for indenting child items in the tree.
    /// This value determines how much horizontal space is used to visually
    /// represent the hierarchical structure of the tree. Each level of
    /// indentation typically represents a deeper level in the tree hierarchy.
    pub indent: usize,
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let symbol = |kind: &Kind| -> &str {
            match kind {
                Kind::Folded { .. } => &self.folded_symbol,
                Kind::Unfolded { .. } => &self.unfolded_symbol,
            }
        };

        let indent = |kind: &Kind| -> usize {
            match kind {
                Kind::Folded { path, .. } | Kind::Unfolded { path, .. } => path.len() * self.indent,
            }
        };

        let id = |kind: &Kind| -> String {
            match kind {
                Kind::Folded { id, .. } | Kind::Unfolded { id, .. } => id.clone(),
            }
        };

        let height = match self.lines {
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
                        self.active_item_style,
                    )
                } else {
                    StyledGraphemes::from_str(
                        format!(
                            "{}{}{}",
                            " ".repeat(StyledGraphemes::from(symbol(kind)).widths()),
                            " ".repeat(indent(kind)),
                            id(kind),
                        ),
                        self.inactive_item_style,
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
