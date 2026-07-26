use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

mod history;
pub use history::History;
#[path = "text_editor/text_editor.rs"]
mod inner;
pub use inner::{Mode, TextEditor};
pub mod config;
pub use config::Config;

#[derive(Clone, Default)]
pub struct State {
    /// The `TextEditor` component to be rendered.
    pub texteditor: TextEditor,
    /// Optional history for navigating through previous inputs.
    pub history: Option<History>,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let mut buf = StyledGraphemes::default();

        let mut styled_prefix =
            StyledGraphemes::from_str(&self.config.prefix, self.config.prefix_style);
        let prefix_width = styled_prefix.widths();

        buf.append(&mut styled_prefix);

        let text = match self.config.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };
        let cursor_column = prefix_width + text.widths_to(self.texteditor.position());

        let mut styled = text
            .apply_style(self.config.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.config.active_char_style);

        buf.append(&mut styled);

        CreatedGraphemes {
            graphemes: buf,
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: Some(ContentPosition {
                row: 0,
                column: cursor_column,
            }),
        }
    }
}

impl State {
    /// Interprets a text-editor content position as a cursor target.
    ///
    /// The prefix occupies content columns but is not editable, so clicking it
    /// resolves to the start of the input. Columns inside wide characters resolve
    /// to that character, and columns beyond the rendered input resolve to the
    /// trailing cursor position. Masked input is measured using the mask that is
    /// actually displayed.
    pub fn hit_at(&self, position: ContentPosition) -> Option<TextEditorHit> {
        if position.row != 0 {
            return None;
        }

        let prefix_width = StyledGraphemes::from(&self.config.prefix).widths();
        let input_column = position.column.saturating_sub(prefix_width);
        let rendered = match self.config.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut start = 0;
        let index = rendered
            .iter()
            .enumerate()
            .find_map(|(index, grapheme)| {
                let end = start + grapheme.width();
                let contains = input_column < end;
                start = end;
                contains.then_some(index)
            })
            .unwrap_or_else(|| rendered.len().saturating_sub(1));

        Some(TextEditorHit::Cursor { index })
    }
}

/// Semantic targets exposed by the text-editor widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextEditorHit {
    Cursor { index: usize },
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn state(text: &str, prefix: &str) -> State {
        State {
            texteditor: TextEditor::new(text),
            config: Config {
                prefix: prefix.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn resolves_prefix_input_and_trailing_columns() {
        let state = state("abc", ">> ");

        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 1 }),
            Some(TextEditorHit::Cursor { index: 0 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 4 }),
            Some(TextEditorHit::Cursor { index: 1 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 80 }),
            Some(TextEditorHit::Cursor { index: 3 })
        );
        assert_eq!(state.hit_at(ContentPosition { row: 1, column: 0 }), None);
    }

    #[test]
    fn resolves_columns_inside_wide_characters() {
        let state = state("界a", "");

        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 0 }),
            Some(TextEditorHit::Cursor { index: 0 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 1 }),
            Some(TextEditorHit::Cursor { index: 0 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 2 }),
            Some(TextEditorHit::Cursor { index: 1 })
        );
    }

    #[test]
    fn uses_the_rendered_mask_width() {
        let mut state = state("界a", "");
        state.config.mask = Some('*');

        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 1 }),
            Some(TextEditorHit::Cursor { index: 1 })
        );
    }

    #[test]
    fn renders_a_multiline_cursor_at_its_logical_position() {
        let state = state("ab\n界c", ">> ");

        let created = state.create_graphemes();

        assert_eq!(">> ab\n界c ", created.graphemes.to_string());
        assert_eq!(Some(ContentPosition { row: 1, column: 3 }), created.cursor);
    }

    #[test]
    fn resolves_clicks_on_each_multiline_row() {
        let state = state("ab\n界c", ">> ");

        assert_eq!(
            state.hit_at(ContentPosition { row: 0, column: 80 }),
            Some(TextEditorHit::Cursor { index: 2 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 0 }),
            Some(TextEditorHit::Cursor { index: 3 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 1 }),
            Some(TextEditorHit::Cursor { index: 3 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 2 }),
            Some(TextEditorHit::Cursor { index: 4 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 80 }),
            Some(TextEditorHit::Cursor { index: 5 })
        );
        assert_eq!(state.hit_at(ContentPosition { row: 2, column: 0 }), None);
    }

    #[test]
    fn resolves_clicks_on_empty_multiline_rows() {
        let state = state("a\n\nb", "");

        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 0 }),
            Some(TextEditorHit::Cursor { index: 2 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 2, column: 0 }),
            Some(TextEditorHit::Cursor { index: 3 })
        );
    }

    #[test]
    fn masking_preserves_multiline_layout_and_cursor_position() {
        let mut state = state("ab\n界c", ">> ");
        state.config.mask = Some('*');

        let created = state.create_graphemes();

        assert_eq!(">> **\n** ", created.graphemes.to_string());
        assert_eq!(Some(ContentPosition { row: 1, column: 2 }), created.cursor);
    }
}
