use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, WidthMode, grapheme::StyledGraphemes,
};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
pub mod yamlz;

/// Represents YAML view state within the application.
#[derive(Clone)]
pub struct State {
    /// The current YAML document being displayed.
    pub document: Document,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let rows = self.document.visible_rows();
        let active_row = self.document.visible_position();
        self.create_graphemes_from_rows(&rows, active_row)
    }

    fn create_graphemes_in_viewport(&self, _width: u16, height: u16) -> CreatedGraphemes {
        let height = self
            .config
            .lines
            .map_or(height as usize, |lines| lines.min(height as usize));
        let rows = self.document.extract_rows_from_current(height);
        self.create_graphemes_from_rows(&rows, 0)
    }
}

impl State {
    fn create_graphemes_from_rows(
        &self,
        rows: &[yamlz::Row],
        active_row: usize,
    ) -> CreatedGraphemes {
        let formatted_rows = self.config.render_content_rows(rows, active_row);

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(formatted_rows),
            layout: WidgetLayout {
                max_height: self.config.lines,
                width_mode: match self.config.overflow_mode {
                    config::OverflowMode::Truncate => WidthMode::Truncate,
                    config::OverflowMode::Wrap => WidthMode::Wrap,
                },
            },
            cursor: (!rows.is_empty()).then_some(ContentPosition {
                row: active_row,
                column: 0,
            }),
        }
    }

    /// Interprets a YAML content position as a semantic operation target.
    ///
    /// The logical content row is resolved back to the underlying document row,
    /// including YAML sequence mappings whose source rows are merged into one
    /// displayed line. Wrapped visual rows are already normalized to their
    /// logical content row by the core renderer.
    pub fn hit_at(&self, position: ContentPosition) -> Option<YamlHit> {
        self.document
            .row_index_at_visible_position(position.row)
            .map(|row_index| YamlHit::Toggle { row_index })
    }

    /// Interprets a content position in a viewport projection as a semantic target.
    pub fn hit_at_viewport(&self, position: ContentPosition) -> Option<YamlHit> {
        self.document
            .row_index_at_visible_offset_from_current(position.row)
            .map(|row_index| YamlHit::Toggle { row_index })
    }
}

/// Semantic targets exposed by the YAML widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YamlHit {
    Toggle { row_index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_full_content_and_resolves_visible_rows_for_hits() {
        let value =
            serde_yaml::from_str("first: one\nsecond:\n  nested: two\nthird: three\n").unwrap();
        let mut state = State {
            document: Document::new([&value]),
            config: Config::default(),
        };

        let initial = state.create_graphemes();
        assert!(initial.graphemes.to_string().contains("first: one"));
        assert!(initial.graphemes.to_string().contains("third: three"));
        assert_eq!(initial.cursor.unwrap().row, 0);

        state.document.down();
        let moved = state.create_graphemes();
        assert!(moved.graphemes.to_string().contains("first: one"));
        assert_eq!(moved.cursor.unwrap().row, 1);

        assert!(matches!(
            state.hit_at(ContentPosition { row: 1, column: 4 }),
            Some(YamlHit::Toggle { .. })
        ));
    }

    #[test]
    fn viewport_projection_is_bounded_and_resolves_hits_from_cursor() {
        let value = serde_yaml::from_str(
            "first: one\nsecond:\n  nested: two\n  extra: value\nthird: three\n",
        )
        .unwrap();
        let mut state = State {
            document: Document::new([&value]),
            config: Config::default(),
        };

        state.document.down();
        let projected = state.create_graphemes_in_viewport(80, 2);
        let rendered = projected.graphemes.to_string();

        assert!(rendered.contains("second: "));
        assert!(rendered.contains("nested: two"));
        assert!(!rendered.contains("extra: value"));
        assert!(!rendered.contains("third: three"));
        assert_eq!(projected.cursor.unwrap().row, 0);

        let YamlHit::Toggle { row_index } = state
            .hit_at_viewport(ContentPosition { row: 0, column: 4 })
            .unwrap();
        state.document.toggle_at(row_index);

        let collapsed = state.create_graphemes_in_viewport(80, 2);
        assert!(collapsed.graphemes.to_string().contains("second: {…}"));
    }

    #[test]
    fn configured_line_limit_bounds_viewport_projection() {
        let value = serde_yaml::from_str("first: one\nsecond: two\n").unwrap();
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
}
