use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{trim, Graphemes},
    item_box::ItemBox,
    pane::Pane,
};

use super::{AsAny, Component};

#[derive(Clone)]
pub struct ItemPicker {
    pub itembox: ItemBox,

    pub style: ContentStyle,
    pub cursor: String,
    pub cursor_style: ContentStyle,
    pub lines: Option<usize>,
}

impl ItemPicker {
    fn itembox_to_layout(&self) -> Vec<Graphemes> {
        self.itembox
            .content()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.itembox.position {
                    Graphemes::new_with_style(format!("{}{}", self.cursor, item), self.cursor_style)
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}",
                            " ".repeat(Graphemes::new(self.cursor.clone()).widths()),
                            item
                        ),
                        self.style,
                    )
                }
            })
            .collect()
    }
}

impl Component for ItemPicker {
    fn make_pane(&self, width: u16) -> Pane {
        let trimed = self
            .itembox_to_layout()
            .iter()
            .map(|row| trim(width as usize, row))
            .collect();
        Pane::new(trimed, self.itembox.position, self.lines)
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
                self.itembox.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.itembox.forward();
            }

            _ => (),
        }
    }

    fn postrun(&mut self) {
        self.itembox.position = 0;
    }
}

impl AsAny for ItemPicker {
    fn as_any(&self) -> &dyn Any {
        self
    }
}