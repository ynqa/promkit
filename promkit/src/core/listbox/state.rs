use crate::{
    crossterm::style::ContentStyle,
    grapheme::{trim, Graphemes, StyledGraphemes},
    impl_as_any,
    pane::Pane,
    PaneFactory,
};

use super::Listbox;

/// Represents the state of a `Listbox` component, including its appearance and behavior.
/// This state includes the currently selected item, styles for active and inactive items,
/// and the number of lines available for rendering the listbox.
#[derive(Clone)]
pub struct State {
    /// The `Listbox` component to be rendered.
    pub listbox: Listbox,

    /// Symbol for the selected line.
    pub cursor: String,

    /// Style for the selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected lines.
    pub inactive_item_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl_as_any!(State);

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let viewport = self.listbox.viewport_range(height);

        let relative_position = self.listbox.position().saturating_sub(viewport.0);

        let matrix = self
            .listbox
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= viewport.0 && *i < viewport.1)
            .map(|(i, item)| {
                if i == self.listbox.position() {
                    StyledGraphemes::from_str(
                        format!("{}{}", self.cursor, item),
                        self.active_item_style,
                    )
                } else {
                    StyledGraphemes::from_str(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::from(self.cursor.clone()).widths()),
                            item
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<StyledGraphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, relative_position)
    }
}
