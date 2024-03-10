use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{trim, Graphemes, StyledGraphemes},
    keymap::KeymapManager,
    pane::Pane,
    AsAny, Result,
};

use super::{Kind, Tree};

/// Represents a renderer for a tree structure,
/// capable of visualizing the tree in a pane.
/// It supports custom symbols for folded and unfolded items,
/// styles for active and inactive items,
/// and a configurable number of lines for rendering.
/// It also handles key events for navigation and folding/unfolding.
#[derive(Clone)]
pub struct Renderer {
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

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
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

        let matrix = self
            .tree
            .kinds()
            .iter()
            .enumerate()
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
                            " ".repeat(Graphemes::from(symbol(kind)).widths()),
                            " ".repeat(indent(kind)),
                            id(kind),
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<StyledGraphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        vec![Pane::new(trimed, self.tree.position(), self.lines)]
    }

    fn postrun(&mut self) {
        self.tree.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
