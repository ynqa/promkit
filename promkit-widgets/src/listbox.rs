use promkit_core::{Widget, grapheme::StyledGraphemes};

#[path = "listbox/listbox.rs"]
mod inner;
pub use inner::Listbox;
pub mod config;
pub use config::Config;

/// Represents the state of a `Listbox` component, including its appearance and behavior.
/// This state includes the currently selected item, styles for active and inactive items,
/// and the number of lines available for rendering the listbox.
#[derive(Clone)]
pub struct State {
    /// The `Listbox` component to be rendered.
    pub listbox: Listbox,
    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self, _width: u16, height: u16) -> StyledGraphemes {
        let height = match self.config.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let lines = self
            .listbox
            .items()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.listbox.position() && *i < self.listbox.position() + height)
            .map(|(i, item)| {
                if i == self.listbox.position() {
                    let init = StyledGraphemes::from_iter([
                        &StyledGraphemes::from(&self.config.cursor),
                        item,
                    ]);
                    if let Some(style) = &self.config.active_item_style {
                        init.apply_style(*style)
                    } else {
                        init
                    }
                } else {
                    let init = StyledGraphemes::from_iter([
                        &StyledGraphemes::from(
                            " ".repeat(StyledGraphemes::from(&self.config.cursor).widths()),
                        ),
                        item,
                    ]);
                    if let Some(style) = &self.config.inactive_item_style {
                        init.apply_style(*style)
                    } else {
                        init
                    }
                }
            });

        StyledGraphemes::from_lines(lines)
    }
}
