use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    error::Result,
    grapheme::{trim, Graphemes},
    listbox::Listbox,
    pane::Pane,
    render::{AsAny, Renderable, State},
};

#[derive(Clone)]
pub struct Renderer {
    pub listbox: Listbox,

    /// Style for selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected line.
    pub inactive_item_style: ContentStyle,

    /// Symbol for selected line.
    pub cursor: String,

    /// Window size.
    pub window_size: Option<usize>,
}

impl State<Renderer> {
    pub fn try_new(
        listbox: Listbox,
        active_item_style: ContentStyle,
        inactive_item_style: ContentStyle,
        cursor: String,
        window_size: Option<usize>,
    ) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(Renderer {
            listbox,
            active_item_style,
            inactive_item_style,
            cursor,
            window_size,
        })))
    }
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
                            " ".repeat(Graphemes::new(self.cursor.clone()).widths()),
                            item
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();

        Pane::new(trimed, self.listbox.position(), self.window_size)
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