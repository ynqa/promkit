use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
pub mod path;
pub mod treez;
pub use treez::Row;

/// Represents the state of a tree structure within the application.
#[derive(Clone)]
pub struct State {
    pub document: Document,
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let symbol = |row: &Row| -> &str {
            if row.has_children && !row.collapsed {
                &self.config.unfolded_symbol
            } else {
                &self.config.folded_symbol
            }
        };

        let rows = self.document.visible_rows();
        let active_row = self.document.visible_position();
        let lines = rows.iter().enumerate().map(|(offset, row)| {
            if offset == active_row {
                StyledGraphemes::from_str(
                    format!(
                        "{}{}{}",
                        symbol(row),
                        " ".repeat(row.depth * self.config.indent),
                        row.id,
                    ),
                    self.config.active_item_style,
                )
            } else {
                StyledGraphemes::from_str(
                    format!(
                        "{}{}{}",
                        " ".repeat(StyledGraphemes::from(symbol(row)).widths()),
                        " ".repeat(row.depth * self.config.indent),
                        row.id,
                    ),
                    self.config.inactive_item_style,
                )
            }
        });

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(lines),
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: (!rows.is_empty()).then_some(ContentPosition {
                row: active_row,
                column: 0,
            }),
        }
    }
}

impl State {
    /// Interprets a tree content position as a semantic operation target.
    ///
    /// Wrapped visual rows are normalized to their logical content row by the
    /// core renderer before this method resolves the underlying document row.
    pub fn hit_at(&self, position: ContentPosition) -> Option<TreeHit> {
        self.document
            .row_index_at_visible_position(position.row)
            .map(|row_index| TreeHit::Toggle { row_index })
    }
}

/// Semantic targets exposed by the tree widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeHit {
    Toggle { row_index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_visible_rows_for_hits() {
        let state = State {
            document: Document::new(vec![
                Row {
                    id: "root".into(),
                    path: vec!["root".into()],
                    depth: 0,
                    has_children: true,
                    collapsed: false,
                },
                Row {
                    id: "child".into(),
                    path: vec!["root".into(), "child".into()],
                    depth: 1,
                    has_children: false,
                    collapsed: false,
                },
            ]),
            config: Config::default(),
        };

        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 20 }),
            Some(TreeHit::Toggle { row_index: 1 })
        );
        assert_eq!(state.hit_at(ContentPosition { row: 2, column: 0 }), None);
    }
}
