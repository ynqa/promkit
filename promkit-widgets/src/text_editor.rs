use promkit_core::{Pane, PaneFactory, grapheme::StyledGraphemes};

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

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let mut buf = StyledGraphemes::default();

        let mut styled_prefix =
            StyledGraphemes::from_str(&self.config.prefix, self.config.prefix_style);

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

        let (matrix, offset) = buf.matrixify(
            width as usize,
            height,
            (StyledGraphemes::from_str(&self.config.prefix, self.config.prefix_style).widths()
                + self.texteditor.position())
                / width as usize,
        );

        Pane::new(matrix, offset)
    }
}
