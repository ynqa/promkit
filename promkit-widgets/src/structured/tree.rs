use promkit_core::{Widget, grapheme::StyledGraphemes};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
pub mod path;
pub use path::create_rows_from_path;
pub mod treez;
pub use treez::Row;

/// Represents the state of a tree structure within the application.
#[derive(Clone)]
pub struct State {
    pub document: Document,
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self, _width: u16, height: u16) -> StyledGraphemes {
        let symbol = |row: &Row| -> &str {
            if row.has_children && !row.collapsed {
                &self.config.unfolded_symbol
            } else {
                &self.config.folded_symbol
            }
        };

        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.document.extract_rows_from_current(height);
        let lines = rows.iter().enumerate().map(|(offset, row)| {
            if offset == 0 {
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

        StyledGraphemes::from_lines(lines)
    }
}
