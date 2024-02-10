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

use super::Checkbox;

#[derive(Clone)]
pub struct Renderer {
    pub checkbox: Checkbox,

    pub style: ContentStyle,
    pub cursor: String,
    pub cursor_style: ContentStyle,
    pub mark: String,
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
                        self.cursor_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::new(self.cursor.clone()).widths()),
                            f(i, item)
                        ),
                        self.style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, self.checkbox.position(), self.lines)
    }

    /// Default key bindings for item picker.
    ///
    /// | Key                    | Description
    /// | :--                    | :--
    /// | <kbd> Enter </kbd>     | Exit the event-loop
    /// | <kbd> CTRL + C </kbd>  | Exit the event-loop with an error
    /// | <kbd> ↑ </kbd>         | Move backward
    /// | <kbd> ↓ </kbd>         | Move forward
    /// | <kbd> CTRL + A </kbd>  | Move to the beginning of the items
    /// | <kbd> CTRL + E </kbd>  | Move to the end of the items
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
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.checkbox.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.checkbox.move_to_tail(),
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
