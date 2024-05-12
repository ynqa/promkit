use crate::{crossterm::style::ContentStyle, grapheme::StyledGraphemes, pane::Pane, PaneFactory};

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
    pub active_item_style: Option<ContentStyle>,
    /// Style for un-selected lines.
    pub inactive_item_style: Option<ContentStyle>,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let matrix = self
            .listbox
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.listbox.position() && *i < self.listbox.position() + height)
            .map(|(i, item)| {
                if i == self.listbox.position() {
                    StyledGraphemes::from_iter([&StyledGraphemes::from(self.cursor.clone()), item])
                } else {
                    let init = StyledGraphemes::from_iter([
                        &StyledGraphemes::from(
                            " ".repeat(StyledGraphemes::from(self.cursor.clone()).widths()),
                        ),
                        item,
                    ]);
                    if let Some(style) = &self.active_item_style {
                        init.apply_style(*style)
                    } else {
                        init
                    }
                }
            })
            .fold((vec![], 0), |(mut acc, pos), item| {
                let rows = item.matrixify(width as usize, height, 0).0;
                if pos < self.listbox.position() + height {
                    acc.extend(rows);
                }
                (acc, pos + 1)
            });

        Pane::new(matrix.0, 0)
    }
}
