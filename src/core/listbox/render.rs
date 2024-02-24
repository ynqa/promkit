use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{trim, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable},
};

use super::Listbox;

/// Represents a renderer for the `Listbox` component,
/// capable of visualizing a list of items in a pane.
/// It supports a custom symbol for the selected line,
/// styles for active and inactive items,
/// and a configurable number of lines for rendering.
#[derive(Clone)]
pub struct Renderer {
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

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let matrix = self
            .listbox
            .items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.listbox.position() {
                    Graphemes::new_with_style(
                        format!("{}{}", self.cursor, item),
                        self.active_item_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::from(self.cursor.clone()).widths()),
                            item
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, self.listbox.position(), self.lines)
    }

    /// Default key bindings for listbox.
    ///
    /// | Key            | Description
    /// | :--            | :--
    /// | <kbd> ↑ </kbd> | Move the cursor backward
    /// | <kbd> ↓ </kbd> | Move the cursor forward
    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.listbox.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.listbox.forward();
            }

            _ => (),
        }
    }

    fn postrun(&mut self) {
        self.listbox.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
