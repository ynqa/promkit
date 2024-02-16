use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    error::Result,
    grapheme::{trim, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable, State},
};

use super::Checkbox;

#[derive(Clone)]
pub struct Renderer {
    pub checkbox: Checkbox,

    /// Style for selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected line.
    pub inactive_item_style: ContentStyle,

    /// Symbol for selected line.
    pub cursor: String,
    /// Checkmark (within [ ] parentheses).
    pub mark: String,

    /// Window size.
    pub window_size: Option<usize>,
}

impl State<Renderer> {
    pub fn try_new(
        checkbox: Checkbox,
        active_item_style: ContentStyle,
        inactive_item_style: ContentStyle,
        cursor: String,
        mark: String,
        window_size: Option<usize>,
    ) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(Renderer {
            checkbox,
            active_item_style,
            inactive_item_style,
            cursor,
            mark,
            window_size,
        })))
    }
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let f = |idx: usize, item: &String| -> String {
            if self.checkbox.picked_indexes().contains(&idx) {
                format!("[{}] {}", self.mark, item)
            } else {
                format!(
                    "[{}] {}",
                    " ".repeat(Graphemes::new(self.mark.clone()).widths()),
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
                            " ".repeat(Graphemes::new(self.cursor.clone()).widths()),
                            f(i, item)
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, self.checkbox.position(), self.window_size)
    }

    /// Default key bindings for item picker.
    ///
    /// | Key                    | Description
    /// | :--                    | :--
    /// | <kbd> Enter </kbd>     | Exit the event-loop
    /// | <kbd> CTRL + C </kbd>  | Exit the event-loop with an error
    /// | <kbd> ↑ </kbd>         | Move backward
    /// | <kbd> ↓ </kbd>         | Move forward
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