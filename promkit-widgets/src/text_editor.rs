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
