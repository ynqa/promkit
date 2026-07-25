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
        let formatted_rows = self.config.render_content_rows(&rows, active_row);

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
}

impl State {
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
}
