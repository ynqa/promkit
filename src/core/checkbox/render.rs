use std::any::Any;

use crate::{
    crossterm::style::ContentStyle,
    grapheme::{trim, Graphemes, StyledGraphemes},
    pane::Pane,
    AsAny,
};

use super::Checkbox;

/// Represents a renderer for the `Checkbox` component,
/// capable of visualizing checkboxes in a pane.
/// It supports custom symbols for the cursor and checkmark,
/// styles for active and inactive items,
/// and a configurable number of lines for rendering.
/// It also handles key events for navigation and toggling checkboxes.
#[derive(Clone)]
pub struct Renderer {
    /// The `Checkbox` component to be rendered.
    pub checkbox: Checkbox,

    /// Symbol for the selected line.
    pub cursor: String,

    /// Symbol used to indicate an active (selected) checkbox item.
    pub active_mark: char,
    /// Symbol used to indicate an inactive (unselected) checkbox item.
    pub inactive_mark: char,

    /// Style for the selected line.
    pub active_item_style: ContentStyle,
    /// Style for unselected lines.
    pub inactive_item_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        let f = |idx: usize, item: &String| -> String {
            if self.checkbox.picked_indexes().contains(&idx) {
                format!("{} {}", self.active_mark, item)
            } else {
                format!("{} {}", self.inactive_mark, item)
            }
        };

        let matrix = self
            .checkbox
            .items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.checkbox.position() {
                    StyledGraphemes::from_str(
                        format!("{}{}", self.cursor, f(i, item)),
                        self.active_item_style,
                    )
                } else {
                    StyledGraphemes::from_str(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::from(self.cursor.clone()).widths()),
                            f(i, item)
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<StyledGraphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        vec![Pane::new(trimed, self.checkbox.position(), self.lines)]
    }

    fn postrun(&mut self) {
        self.checkbox.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
