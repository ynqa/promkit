use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, WidthMode, grapheme::StyledGraphemes,
};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
mod deserializer;
pub mod jsonz;

use crate::structured::PrettyRender;

/// Represents JSON view state within the application.
///
/// This struct holds the current JSON document and provides
/// methods to navigate and manipulate rows according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    /// The current JSON document being displayed.
    pub document: Document,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let rows = self.document.visible_rows();
        let active_row = self.document.visible_position();
        let line_numbers = self
            .config
            .show_line_numbers
            .then(|| self.document.visible_line_numbers());
        self.create_graphemes_from_rows(&rows, active_row, line_numbers)
    }

    fn create_graphemes_in_viewport(&self, _width: u16, height: u16) -> CreatedGraphemes {
        let height = self
            .config
            .lines
            .map_or(height as usize, |lines| lines.min(height as usize));
        let (viewport_start, active_row) = self.document.project_viewport(height);
        let rows = self.document.extract_rows_from(viewport_start, height);
        let line_numbers = self.config.show_line_numbers.then(|| {
            self.document
                .line_numbers_from_viewport(viewport_start, height)
        });
        self.create_graphemes_from_rows(&rows, active_row, line_numbers)
    }
}

impl State {
    fn create_graphemes_from_rows(
        &self,
        rows: &[jsonz::Row],
        active_row: usize,
        line_numbers: Option<Vec<usize>>,
    ) -> CreatedGraphemes {
        let mut formatted_rows = self.config.render_content_rows(rows, active_row);
        if let Some(line_numbers) = line_numbers {
            formatted_rows =
                super::with_line_numbers(formatted_rows, line_numbers, self.document.line_count());
        }

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(formatted_rows),
            layout: WidgetLayout {
                max_height: self.config.lines,
                width_mode: match self.config.overflow_mode {
                    config::OverflowMode::Truncate => WidthMode::Truncate,
                    config::OverflowMode::Wrap => WidthMode::Wrap,
                },
                ..Default::default()
            },
            cursor: (!rows.is_empty()).then_some(ContentPosition {
                row: active_row,
                column: 0,
            }),
        }
    }

    /// Formats the raw JSON data into a pretty-printed string with indentation.
    pub fn render_pretty_json(&self) -> String {
        self.document.rows().render_pretty(self.config.indent)
    }

    /// Interprets a JSON content position as a semantic operation target.
    ///
    /// Wrapped visual rows are normalized to their logical content row by the
    /// core renderer before this method resolves the underlying document row.
    pub fn hit_at(&self, position: ContentPosition) -> Option<JsonHit> {
        self.document
            .row_index_at_visible_position(position.row)
            .map(|row_index| JsonHit::Toggle { row_index })
    }

    /// Interprets a content position in a viewport projection as a semantic target.
    pub fn hit_at_viewport(&self, position: ContentPosition) -> Option<JsonHit> {
        self.document
            .row_index_at_viewport_position(position.row)
            .map(|row_index| JsonHit::Toggle { row_index })
    }
}

/// Semantic targets exposed by the JSON widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonHit {
    Toggle { row_index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state {
        use super::*;

        mod create_graphemes_in_viewport {
            use super::*;

            #[test]
            fn viewport_projection_is_bounded_and_resolves_hits() {
                let value = serde_json::json!({
                    "first": "one",
                    "second": {
                        "nested": "two"
                    },
                    "third": "three"
                });
                let mut state = State {
                    document: Document::new([&value]),
                    config: Config::default(),
                };

                state.create_graphemes_in_viewport(80, 2);
                state.document.down();
                state.create_graphemes_in_viewport(80, 2);
                state.document.down();
                let projected = state.create_graphemes_in_viewport(80, 2);
                let rendered = projected.graphemes.to_string();

                assert!(rendered.contains("\"first\": \"one\""));
                assert!(rendered.contains("\"second\": {"));
                assert!(!rendered.contains("\"nested\": \"two\""));
                assert!(!rendered.contains("\"third\": \"three\""));
                assert_eq!(projected.cursor.unwrap().row, 1);

                let JsonHit::Toggle { row_index } = state
                    .hit_at_viewport(ContentPosition { row: 1, column: 4 })
                    .unwrap();
                state.document.toggle_at(row_index);

                let collapsed = state.create_graphemes_in_viewport(80, 2);
                assert!(collapsed.graphemes.to_string().contains("\"second\": {…}"));
            }

            #[test]
            fn viewport_projection_stays_stable_until_cursor_leaves_it() {
                let value = serde_json::json!({
                    "first": 1,
                    "second": 2,
                    "third": 3,
                    "fourth": 4
                });
                let mut state = State {
                    document: Document::new([&value]),
                    config: Config::default(),
                };

                let initial = state.create_graphemes_in_viewport(80, 3);
                assert!(initial.graphemes.to_string().starts_with("{"));
                assert_eq!(initial.cursor.unwrap().row, 0);

                state.document.down();
                let moved_inside = state.create_graphemes_in_viewport(80, 3);
                assert!(moved_inside.graphemes.to_string().starts_with("{"));
                assert_eq!(moved_inside.cursor.unwrap().row, 1);

                state.document.down();
                let moved_to_edge = state.create_graphemes_in_viewport(80, 3);
                assert!(moved_to_edge.graphemes.to_string().starts_with("{"));
                assert_eq!(moved_to_edge.cursor.unwrap().row, 2);

                state.document.down();
                let moved_outside = state.create_graphemes_in_viewport(80, 3);
                assert!(
                    moved_outside
                        .graphemes
                        .to_string()
                        .starts_with("\"first\": 1")
                );
                assert_eq!(moved_outside.cursor.unwrap().row, 2);

                state.document.up();
                let moved_back_inside = state.create_graphemes_in_viewport(80, 3);
                assert!(
                    moved_back_inside
                        .graphemes
                        .to_string()
                        .starts_with("\"first\": 1")
                );
                assert_eq!(moved_back_inside.cursor.unwrap().row, 1);

                state.document.up();
                state.create_graphemes_in_viewport(80, 3);
                state.document.up();
                let moved_above = state.create_graphemes_in_viewport(80, 3);
                assert!(moved_above.graphemes.to_string().starts_with("{"));
                assert_eq!(moved_above.cursor.unwrap().row, 0);
            }

            #[test]
            fn configured_line_limit_bounds_viewport_projection() {
                let value = serde_json::json!({"first": 1, "second": 2});
                let state = State {
                    document: Document::new([&value]),
                    config: Config {
                        lines: Some(1),
                        ..Config::default()
                    },
                };

                let projected = state.create_graphemes_in_viewport(80, 20);
                assert_eq!(projected.graphemes.logical_lines().len(), 1);
            }

            #[test]
            fn viewport_projection_uses_stable_line_numbers() {
                let value = serde_json::json!({"first": 1, "second": 2, "third": 3});
                let mut state = State {
                    document: Document::new([&value]),
                    config: Config {
                        show_line_numbers: true,
                        ..Default::default()
                    },
                };

                state.create_graphemes_in_viewport(80, 2);
                state.document.down();
                state.document.down();
                state.document.down();

                assert_eq!(
                    state
                        .create_graphemes_in_viewport(80, 2)
                        .graphemes
                        .to_string(),
                    "3 \"second\": 2,\n4 \"third\": 3"
                );
            }
        }

        mod create_graphemes {
            use super::*;

            #[test]
            fn preserves_expanded_line_numbers_after_toggle() {
                let value = serde_json::json!({
                    "first": {
                        "nested": 1
                    },
                    "last": 2
                });
                let mut state = State {
                    document: Document::new([&value]),
                    config: Config {
                        indent: 2,
                        show_line_numbers: true,
                        ..Default::default()
                    },
                };

                assert_eq!(
                    state.document.visible_line_numbers(),
                    vec![1, 2, 3, 4, 5, 6]
                );

                state.document.down();
                state.document.toggle();

                assert_eq!(state.document.visible_line_numbers(), vec![1, 2, 5, 6]);
                assert_eq!(
                    state.create_graphemes().graphemes.to_string(),
                    "1 {\n2   \"first\": {…},\n5   \"last\": 2\n6 }"
                );
            }
        }
    }
}
