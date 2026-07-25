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
