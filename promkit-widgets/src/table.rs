use std::collections::VecDeque;

use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, WidthMode,
    crossterm::style::ContentStyle,
    grapheme::{StyledGrapheme, StyledGraphemes},
};
use unicode_width::UnicodeWidthChar;

pub mod config;
pub use config::Config;
mod document;
pub use document::{CsvOptions, Document};
use document::{Projection, display_char, display_width};

/// State projected by the table widget.
#[derive(Clone)]
pub struct State {
    /// Compact table document and navigation state.
    pub document: Document,
    /// Rendering configuration.
    pub config: Config,
}

impl State {
    /// Creates table state with the default rendering configuration.
    pub fn new(document: Document) -> Self {
        Self {
            document,
            config: Config::default(),
        }
    }

    /// Resolves a position from the most recent viewport projection.
    pub fn hit_at_viewport(&self, position: ContentPosition) -> Option<TableHit> {
        let projection = self.document.projection();
        if projection.width == 0 {
            return None;
        }

        let row = if projection.header_visible && position.row == 0 {
            None
        } else {
            let body_row = position
                .row
                .checked_sub(usize::from(projection.header_visible))?;
            if body_row >= projection.body_height {
                return None;
            }
            Some(projection.first_row + body_row)
        };

        let column = self.column_at(position.column, projection)?;
        Some(match row {
            Some(row) if row < self.document.row_count() => TableHit::Cell { row, column },
            None => TableHit::Header { column },
            Some(_) => return None,
        })
    }

    fn column_at(&self, target: usize, projection: Projection) -> Option<usize> {
        let separator_width = display_width(&self.config.separator);
        let mut used = 0usize;

        for column in projection.first_column..self.document.column_count() {
            if column != projection.first_column {
                if used.saturating_add(separator_width) > projection.width {
                    break;
                }
                if target < used + separator_width {
                    return None;
                }
                used += separator_width;
            }

            let available = projection.width.saturating_sub(used);
            if available == 0 {
                break;
            }
            let column_width = self.effective_column_width(column).min(available);
            if target < used + column_width {
                return Some(column);
            }
            used += column_width;
        }
        None
    }

    fn effective_column_width(&self, column: usize) -> usize {
        let width = self.document.column_width(column).unwrap_or(0).max(1);
        self.config
            .max_column_width
            .map_or(width, |maximum| width.min(maximum.max(1)))
    }

    fn project(&self, width: usize, height: usize) -> CreatedGraphemes {
        if width == 0 || height == 0 || self.document.column_count() == 0 {
            self.document.set_projection(Projection {
                width,
                ..Projection::default()
            });
            return CreatedGraphemes {
                layout: WidgetLayout {
                    max_height: self.config.lines,
                    width_mode: WidthMode::Truncate,
                },
                ..CreatedGraphemes::default()
            };
        }

        let header_visible = self.document.has_header();
        let body_height = height.saturating_sub(usize::from(header_visible));
        let rows = self.document.projected_rows(body_height);
        let projection = Projection {
            first_row: rows.start,
            first_column: self.document.first_column(),
            body_height: rows.len(),
            width,
            header_visible,
        };
        self.document.set_projection(projection);

        let rendered_rows = usize::from(header_visible) + rows.len();
        let content_width = self.projected_content_width(width);
        let mut graphemes = StyledGraphemes(VecDeque::with_capacity(
            content_width
                .saturating_add(1)
                .saturating_mul(rendered_rows),
        ));

        if header_visible {
            self.render_row(&mut graphemes, None, width, self.config.header_style);
            if !rows.is_empty() {
                graphemes.push_back(StyledGrapheme::from('\n'));
            }
        }

        for (index, row) in rows.clone().enumerate() {
            let mut style = self.config.cell_style;
            if row == self.document.position() {
                style.attributes.set(self.config.active_item_attribute);
            }
            self.render_row(&mut graphemes, Some(row), width, style);
            if index + 1 < rows.len() {
                graphemes.push_back(StyledGrapheme::from('\n'));
            }
        }

        let cursor = (!rows.is_empty()).then_some(ContentPosition {
            row: usize::from(header_visible) + self.document.position() - rows.start,
            column: 0,
        });

        CreatedGraphemes {
            graphemes,
            layout: WidgetLayout {
                max_height: self.config.lines,
                width_mode: WidthMode::Truncate,
            },
            cursor,
        }
    }

    fn render_row(
        &self,
        output: &mut StyledGraphemes,
        row: Option<usize>,
        width: usize,
        style: ContentStyle,
    ) {
        let separator_width = display_width(&self.config.separator);
        let mut used = 0usize;

        for column in self.document.first_column()..self.document.column_count() {
            if column != self.document.first_column() {
                if used.saturating_add(separator_width) > width {
                    break;
                }
                push_text(
                    output,
                    &self.config.separator,
                    separator_width,
                    self.config.separator_style,
                );
                used += separator_width;
            }

            let available = width.saturating_sub(used);
            if available == 0 {
                break;
            }
            let column_width = self.effective_column_width(column).min(available);
            let value = match row {
                Some(row) => self.document.cell(row, column),
                None => self.document.header_cell(column),
            }
            .unwrap_or("");
            let value_width = push_text(output, value, column_width, style);
            for _ in value_width..column_width {
                output.push_back(StyledGrapheme::new(' ', style));
            }
            used += column_width;
        }
    }

    fn projected_content_width(&self, width: usize) -> usize {
        let separator_width = display_width(&self.config.separator);
        let mut used = 0usize;

        for column in self.document.first_column()..self.document.column_count() {
            if column != self.document.first_column() {
                let Some(next) = used.checked_add(separator_width) else {
                    return width;
                };
                if next > width {
                    break;
                }
                used = next;
            }

            let available = width.saturating_sub(used);
            if available == 0 {
                break;
            }
            used = used.saturating_add(self.effective_column_width(column).min(available));
        }
        used
    }
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let height = self
            .config
            .lines
            .unwrap_or_else(|| self.document.row_count() + usize::from(self.document.has_header()));
        self.project(usize::MAX, height)
    }

    fn create_graphemes_in_viewport(&self, width: u16, height: u16) -> CreatedGraphemes {
        let height = self
            .config
            .lines
            .map_or(height as usize, |lines| lines.min(height as usize));
        self.project(width as usize, height)
    }
}

/// Semantic target in the latest table viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableHit {
    /// A header cell.
    Header { column: usize },
    /// A body cell.
    Cell { row: usize, column: usize },
}

fn push_text(
    output: &mut StyledGraphemes,
    value: &str,
    width: usize,
    style: ContentStyle,
) -> usize {
    if width == 0 {
        return 0;
    }

    let full_width = display_width(value);
    let truncated = full_width > width;
    let content_limit = if truncated {
        width.saturating_sub(1)
    } else {
        width
    };
    let mut used = 0usize;

    for ch in value.chars() {
        let ch = display_char(ch);
        let ch_width = ch.width().unwrap_or(0);
        if used.saturating_add(ch_width) > content_limit {
            break;
        }
        output.push_back(StyledGrapheme::new(ch, style));
        used += ch_width;
    }

    if truncated {
        output.push_back(StyledGrapheme::new('…', style));
        used += 1;
    }
    used
}

#[cfg(test)]
mod tests {
    use promkit_core::Widget;

    use super::*;

    fn state(input: &str) -> State {
        State::new(Document::from_csv(input.as_bytes(), CsvOptions::default()).unwrap())
    }

    #[test]
    fn parses_quoted_and_multiline_cells_without_per_cell_ownership() {
        let document = Document::from_csv(
            "name,note\nalice,\"hello, world\"\nbob,\"line 1\nline 2\"\n".as_bytes(),
            CsvOptions::default(),
        )
        .unwrap();

        assert_eq!(document.row_count(), 2);
        assert_eq!(document.column_count(), 2);
        assert_eq!(document.header_cell(1), Some("note"));
        assert_eq!(document.cell(0, 1), Some("hello, world"));
        assert_eq!(document.cell(1, 1), Some("line 1\nline 2"));
    }

    #[test]
    fn supports_a_non_comma_delimiter() {
        let document = Document::from_csv(
            "name\tvalue\nfirst\tone\n".as_bytes(),
            CsvOptions::default().delimiter(b'\t'),
        )
        .unwrap();

        assert_eq!(document.cell(0, 1), Some("one"));
    }

    #[test]
    fn vertical_projection_is_bounded_and_follows_the_cursor() {
        let mut state = state("id,value\n1,one\n2,two\n3,three\n4,four\n");

        let initial = state.create_graphemes_in_viewport(40, 3);
        assert_eq!(initial.graphemes.logical_lines().len(), 3);
        assert!(initial.graphemes.to_string().contains("1"));
        assert!(initial.graphemes.to_string().contains("2"));

        state.document.down();
        state.document.down();
        let moved = state.create_graphemes_in_viewport(40, 3);
        assert!(!moved.graphemes.to_string().contains("1   "));
        assert!(moved.graphemes.to_string().contains("3"));
        assert_eq!(moved.cursor.unwrap().row, 2);
    }

    #[test]
    fn horizontal_projection_scrolls_by_column() {
        let mut state = state("a,b,c\none,two,three\n");

        let first = state.create_graphemes_in_viewport(20, 2);
        assert!(first.graphemes.to_string().contains('a'));

        state.document.scroll_right();
        let second = state.create_graphemes_in_viewport(20, 2);
        assert!(!second.graphemes.to_string().starts_with('a'));
        assert!(second.graphemes.to_string().starts_with('b'));
    }

    #[test]
    fn projection_replaces_embedded_newlines_and_respects_width() {
        let mut state = state("name,note\nalice,\"line 1\nline 2\"\n");
        state.document.scroll_right();
        let projected = state.create_graphemes_in_viewport(10, 2);

        assert_eq!(projected.graphemes.logical_lines().len(), 2);
        assert!(projected.graphemes.to_string().contains('↵'));
        assert!(
            projected
                .graphemes
                .logical_lines()
                .iter()
                .all(|line| line.widths() <= 10)
        );
    }

    #[test]
    fn resolves_header_and_body_hits_in_the_latest_viewport() {
        let mut state = state("a,b,c\none,two,three\n");
        state.document.scroll_right();
        state.create_graphemes_in_viewport(20, 2);

        assert_eq!(
            state.hit_at_viewport(ContentPosition { row: 0, column: 0 }),
            Some(TableHit::Header { column: 1 })
        );
        assert_eq!(
            state.hit_at_viewport(ContentPosition { row: 1, column: 0 }),
            Some(TableHit::Cell { row: 0, column: 1 })
        );
    }

    #[test]
    fn rejects_non_rectangular_csv() {
        assert!(
            Document::from_csv("a,b\none,two\nthree\n".as_bytes(), CsvOptions::default()).is_err()
        );
    }
}
