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
        if target >= projection.width {
            return None;
        }
        let separator_width = display_width(&self.config.separator);
        let target = projection.horizontal_offset.checked_add(target)?;
        let mut start = 0usize;

        for column in 0..self.document.column_count() {
            let end = start.saturating_add(self.document.column_width(column)?);
            if target < end {
                return Some(column);
            }
            start = end;

            if column + 1 < self.document.column_count() {
                let separator_end = start.saturating_add(separator_width);
                if target < separator_end {
                    return None;
                }
                start = separator_end;
            }
        }
        None
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
        let separator_width = display_width(&self.config.separator);
        let content_width = self.document.content_width(separator_width);
        let max_horizontal_offset = content_width.saturating_sub(width);
        let horizontal_offset = self.document.horizontal_offset().min(max_horizontal_offset);
        self.document.set_horizontal_offset(horizontal_offset);
        let projection = Projection {
            first_row: rows.start,
            horizontal_offset,
            max_horizontal_offset,
            body_height: rows.len(),
            width,
            header_visible,
        };
        self.document.set_projection(projection);

        let rendered_rows = usize::from(header_visible) + rows.len();
        let projected_width = content_width.saturating_sub(horizontal_offset).min(width);
        let mut graphemes = StyledGraphemes(VecDeque::with_capacity(
            projected_width
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
        let viewport_start = self.document.horizontal_offset();
        let viewport_end = viewport_start.saturating_add(width);
        let mut segment_start = 0usize;

        for column in 0..self.document.column_count() {
            if segment_start >= viewport_end {
                break;
            }
            let column_width = self.document.column_width(column).unwrap_or(1);
            let value = match row {
                Some(row) => self.document.cell(row, column),
                None => self.document.header_cell(column),
            }
            .unwrap_or("");
            push_text_range(
                output,
                value,
                column_width,
                segment_start,
                viewport_start,
                viewport_end,
                style,
            );
            segment_start = segment_start.saturating_add(column_width);

            if column + 1 < self.document.column_count() {
                push_text_range(
                    output,
                    &self.config.separator,
                    separator_width,
                    segment_start,
                    viewport_start,
                    viewport_end,
                    self.config.separator_style,
                );
                segment_start = segment_start.saturating_add(separator_width);
            }
        }
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

fn push_text_range(
    output: &mut StyledGraphemes,
    value: &str,
    segment_width: usize,
    segment_start: usize,
    viewport_start: usize,
    viewport_end: usize,
    style: ContentStyle,
) {
    let segment_end = segment_start.saturating_add(segment_width);
    if segment_end <= viewport_start || segment_start >= viewport_end {
        return;
    }

    let visible_start = viewport_start.saturating_sub(segment_start);
    let visible_end = viewport_end
        .saturating_sub(segment_start)
        .min(segment_width);

    if value.is_ascii() {
        for position in visible_start..visible_end {
            let ch = value
                .as_bytes()
                .get(position)
                .map_or(' ', |byte| display_char(char::from(*byte)));
            output.push_back(StyledGrapheme::new(ch, style));
        }
        return;
    }

    let mut position = 0usize;
    for ch in value.chars() {
        let ch = display_char(ch);
        let ch_width = ch.width().unwrap_or(0);
        if ch_width == 0 {
            if position > visible_start && position <= visible_end {
                output.push_back(StyledGrapheme::new(ch, style));
            }
            continue;
        }

        let end = position.saturating_add(ch_width);
        if end > visible_start && position < visible_end {
            if position >= visible_start && end <= visible_end {
                output.push_back(StyledGrapheme::new(ch, style));
            } else {
                let overlap_start = position.max(visible_start);
                let overlap_end = end.min(visible_end);
                for _ in overlap_start..overlap_end {
                    output.push_back(StyledGrapheme::new(' ', style));
                }
            }
        }
        position = end;
        if position >= visible_end {
            break;
        }
    }

    for _ in position.max(visible_start)..visible_end {
        output.push_back(StyledGrapheme::new(' ', style));
    }
}

#[cfg(test)]
mod tests {
    use promkit_core::Widget;

    use super::*;

    fn state(input: &str) -> State {
        State::new(Document::from_csv(input.as_bytes(), CsvOptions::default()).unwrap())
    }

    mod document {
        use super::*;

        mod from_csv {
            use super::*;

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
            fn rejects_non_rectangular_input() {
                assert!(
                    Document::from_csv("a,b\none,two\nthree\n".as_bytes(), CsvOptions::default())
                        .is_err()
                );
            }
        }
    }

    mod state {
        use super::*;

        mod create_graphemes_in_viewport {
            use super::*;

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
            fn horizontal_projection_scrolls_by_display_cell() {
                let mut state = state("abc,def\none,two\n");
                state.config.separator = "|".to_owned();

                let first = state.create_graphemes_in_viewport(2, 2);
                assert!(first.graphemes.to_string().starts_with("ab"));

                state.document.scroll_right_by(1);
                let second = state.create_graphemes_in_viewport(2, 2);
                assert!(second.graphemes.to_string().starts_with("bc"));
                assert_eq!(state.document.horizontal_offset(), 1);
            }

            #[test]
            fn horizontal_projection_preserves_cell_suffixes_without_ellipsis() {
                let mut state = state("value\nabcdefghijklmnopqrstuvwxyz\n");
                let initial = state.create_graphemes_in_viewport(10, 2);
                assert!(!initial.graphemes.to_string().contains('…'));

                assert!(state.document.scroll_right_by(16));
                let scrolled = state.create_graphemes_in_viewport(10, 2);
                let lines = scrolled.graphemes.logical_lines();
                assert_eq!(lines[1].to_string(), "qrstuvwxyz");
                assert!(!scrolled.graphemes.to_string().contains('…'));
            }

            #[test]
            fn projection_replaces_embedded_newlines_and_respects_width() {
                let mut state = state("name,note\nalice,\"line 1\nline 2\"\n");
                state.create_graphemes_in_viewport(10, 2);
                state.document.scroll_to_end();
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
        }

        mod hit_at_viewport {
            use super::*;

            #[test]
            fn resolves_header_and_body_in_the_latest_viewport() {
                let mut state = state("a,b,c\nx,y,z\n");
                state.config.separator = "|".to_owned();
                state.create_graphemes_in_viewport(3, 2);
                state.document.scroll_right_by(2);
                state.create_graphemes_in_viewport(3, 2);

                assert_eq!(
                    state.hit_at_viewport(ContentPosition { row: 0, column: 0 }),
                    Some(TableHit::Header { column: 1 })
                );
                assert_eq!(
                    state.hit_at_viewport(ContentPosition { row: 1, column: 0 }),
                    Some(TableHit::Cell { row: 0, column: 1 })
                );
            }
        }
    }
}
