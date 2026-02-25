use promkit_core::{Pane, PaneFactory, grapheme::StyledGraphemes};

#[path = "checkbox/checkbox.rs"]
mod inner;
pub use inner::Checkbox;
pub mod config;
pub use config::Config;

/// Represents the state of a `Checkbox` component.
///
/// This state includes not only the checkbox itself but also various attributes
/// that determine how the checkbox and its items are displayed. These attributes
/// include symbols for indicating active and inactive items, styles for selected
/// and unselected lines, and the number of lines available for rendering.
#[derive(Clone)]
pub struct State {
    /// The `Checkbox` component to be rendered.
    pub checkbox: Checkbox,

    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let f = |idx: usize| -> StyledGraphemes {
            if self.checkbox.picked_indexes().contains(&idx) {
                StyledGraphemes::from(format!("{} ", self.config.active_mark))
            } else {
                StyledGraphemes::from(format!("{} ", self.config.inactive_mark))
            }
        };

        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let matrix = self
            .checkbox
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                *i >= self.checkbox.position() && *i < self.checkbox.position() + height
            })
            .map(|(i, item)| {
                if i == self.checkbox.position() {
                    StyledGraphemes::from_iter([
                        &StyledGraphemes::from(&self.config.cursor),
                        &f(i),
                        item,
                    ])
                    .apply_style(self.config.active_item_style)
                } else {
                    StyledGraphemes::from_iter([
                        &StyledGraphemes::from(
                            " ".repeat(StyledGraphemes::from(&self.config.cursor).widths()),
                        ),
                        &f(i),
                        item,
                    ])
                    .apply_style(self.config.inactive_item_style)
                }
            })
            .fold((vec![], 0), |(mut acc, pos), item| {
                let rows = item.matrixify(width as usize, height, 0).0;
                if pos < self.checkbox.position() + height {
                    acc.extend(rows);
                }
                (acc, pos + 1)
            });

        Pane::new(matrix.0, 0)
    }
}
