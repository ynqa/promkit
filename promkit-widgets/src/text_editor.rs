use promkit_core::{Widget, grapheme::StyledGraphemes};

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
    fn create_graphemes(&self, width: u16, height: u16) -> StyledGraphemes {
        if width == 0 {
            return StyledGraphemes::default();
        }

        let mut buf = StyledGraphemes::default();

        let mut styled_prefix =
            StyledGraphemes::from_str(&self.config.prefix, self.config.prefix_style);
        let prefix_width = styled_prefix.widths();

        buf.append(&mut styled_prefix);

        let text = match self.config.mask {
            Some(mask) => self.texteditor.masking(mask),
            None => self.texteditor.text(),
        };

        let mut styled = text
            .apply_style(self.config.inactive_char_style)
            .apply_style_at(self.texteditor.position(), self.config.active_char_style);

        buf.append(&mut styled);

        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = buf.wrapped_lines(width as usize);
        if rows.is_empty() || height == 0 {
            return StyledGraphemes::default();
        }

        let lines = rows.len().min(height);
        let mut start = (prefix_width + self.texteditor.position()) / width as usize;
        let end = start + lines;
        if end > rows.len() {
            start = rows.len().saturating_sub(lines);
        }

        StyledGraphemes::from_lines(rows.into_iter().skip(start).take(lines))
    }
}
