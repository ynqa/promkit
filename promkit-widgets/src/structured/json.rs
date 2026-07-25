use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, WidthMode, grapheme::StyledGraphemes,
};

mod document;
pub use document::Document;
pub mod config;
pub use config::Config;
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
}

/// Semantic targets exposed by the JSON widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonHit {
    Toggle { row_index: usize },
}
