use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{trim, Grapheme, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable},
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
    /// Checkmark symbol (displayed within square brackets).
    pub mark: char,

    /// Style for the selected line.
    pub active_item_style: ContentStyle,
    /// Style for unselected lines.
    pub inactive_item_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let f = |idx: usize, item: &String| -> String {
            if self.checkbox.picked_indexes().contains(&idx) {
                format!("[{}] {}", self.mark, item)
            } else {
                format!(
                    "[{}] {}",
                    " ".repeat(Grapheme::new(self.mark).width()),
                    item
                )
            }
        };

        let matrix = self
            .checkbox
            .items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.checkbox.position() {
                    Graphemes::new_with_style(
                        format!("{}{}", self.cursor, f(i, item)),
                        self.active_item_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::from(self.cursor.clone()).widths()),
                            f(i, item)
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, self.checkbox.position(), self.lines)
    }

    /// Default key bindings for checkbox.
    ///
    /// | Key                | Description
    /// | :--                | :--
    /// | <kbd> ↑ </kbd>     | Move the cursor backward
    /// | <kbd> ↓ </kbd>     | Move the cursor forward
    /// | <kbd> Space </kbd> | Put checkmark for the current item
    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.checkbox.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.checkbox.forward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.checkbox.toggle(),

            _ => (),
        }
    }

    fn postrun(&mut self) {
        self.checkbox.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
