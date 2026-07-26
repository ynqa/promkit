use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

mod history;
pub use history::History;
#[path = "text_editor/text_editor.rs"]
mod inner;
pub use inner::{Mode, TextEditor, TextPosition};
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
        let continuation_prefix =
            StyledGraphemes::from_str(&self.config.continuation_prefix, self.config.prefix_style);
        let continuation_prefix_width = continuation_prefix.widths();

        buf.append(&mut styled_prefix);

        let text = match self.config.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };
        let cursor = self.texteditor.logical_position();
        let rendered_cursor_column =
            text.iter()
                .take(self.texteditor.position())
                .fold(0, |column, grapheme| {
                    if grapheme.character() == '\n' {
                        0
                    } else {
                        column + grapheme.width()
                    }
                });
        let cursor_column = rendered_cursor_column
            + if cursor.row == 0 {
                prefix_width
            } else {
                continuation_prefix_width
            };

        let mut text = text.apply_style(self.config.inactive_char_style);
        let cursor_index = self.texteditor.position();
        if text
            .iter()
            .nth(cursor_index)
            .is_some_and(|grapheme| grapheme.character() == '\n')
        {
            text.insert(
                cursor_index,
                promkit_core::grapheme::StyledGrapheme::from(' '),
            );
        }
        let styled = text.apply_style_at(cursor_index, self.config.active_char_style);
        let mut rendered = StyledGraphemes::default();
        for grapheme in styled.iter() {
            rendered.push_back(grapheme.clone());
            if grapheme.character() == '\n' {
                rendered.append(&mut continuation_prefix.clone());
            }
        }

        buf.append(&mut rendered);

        CreatedGraphemes {
            graphemes: buf,
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: Some(ContentPosition {
                row: cursor.row,
                column: cursor_column,
            }),
        }
    }
}

impl State {
    /// Interprets a text-editor content position as a cursor target.
    ///
    /// The first-row and continuation prefixes occupy content columns but are
    /// not editable, so clicking either resolves to the start of its input row.
    /// Columns inside wide characters resolve to that character, and columns
    /// beyond the rendered input resolve to the trailing cursor position. Masked
    /// input is measured using the mask that is actually displayed.
    pub fn hit_at(&self, position: ContentPosition) -> Option<TextEditorHit> {
        let row_prefix = if position.row == 0 {
            &self.config.prefix
        } else {
            &self.config.continuation_prefix
        };
        let prefix_width = StyledGraphemes::from(row_prefix).widths();
        let input_column = position.column.saturating_sub(prefix_width);

        if self.config.mask.is_some() {
            let text = self.texteditor.text_without_cursor();
            let mut lines = text.logical_lines();
            if lines.is_empty() {
                lines.push(StyledGraphemes::default());
            }
            let line = lines.get(position.row)?;
            let start = lines
                .iter()
                .take(position.row)
                .map(|line| line.len() + 1)
                .sum::<usize>();

            return Some(TextEditorHit::Cursor {
                index: start + input_column.min(line.len()),
            });
        }

        self.texteditor
            .position_at(TextPosition {
                row: position.row,
                column: input_column,
            })
            .map(|index| TextEditorHit::Cursor { index })
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
    fn renders_a_visible_cursor_before_a_newline() {
        let mut state = state("ab\ncd", "");
        assert!(state.texteditor.move_to(2));

        let created = state.create_graphemes();

        assert_eq!("ab \ncd ", created.graphemes.to_string());
        assert_eq!(Some(ContentPosition { row: 0, column: 2 }), created.cursor);
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

    #[test]
    fn renders_a_continuation_prefix_without_changing_the_editor_text() {
        let mut state = state("first\nsecond", ">>> ");
        state.config.continuation_prefix = "... ".into();

        let created = state.create_graphemes();

        assert_eq!(
            state.texteditor.text_without_cursor().to_string(),
            "first\nsecond"
        );
        assert_eq!(created.graphemes.to_string(), ">>> first\n... second ");
        assert_eq!(created.cursor, Some(ContentPosition { row: 1, column: 10 }));
    }

    #[test]
    fn uses_the_continuation_prefix_width_for_multiline_hits() {
        let mut state = state("first\nsecond", ">>> ");
        state.config.continuation_prefix = "... ".into();

        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 2 }),
            Some(TextEditorHit::Cursor { index: 6 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 5 }),
            Some(TextEditorHit::Cursor { index: 7 })
        );
        assert_eq!(
            state.hit_at(ContentPosition { row: 1, column: 80 }),
            Some(TextEditorHit::Cursor { index: 12 })
        );
    }
}
