use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{trim, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable},
    tree::Tree,
};

#[derive(Clone)]
pub struct Renderer {
    pub tree: Tree,

    pub style: ContentStyle,
    pub cursor: String,
    pub cursor_style: ContentStyle,
    pub lines: Option<usize>,
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> crate::pane::Pane {
        let matrix = self
            .tree
            .nodes_with_depth()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.tree.position {
                    Graphemes::new_with_style(
                        format!("{}{}{}", self.cursor, " ".repeat(item.depth), item.data),
                        self.cursor_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}{}",
                            " ".repeat(Graphemes::new(self.cursor.clone()).widths()),
                            " ".repeat(item.depth),
                            item.data
                        ),
                        self.style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        Pane::new(trimed, self.tree.position, self.lines)
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.tree.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.tree.forward();
            }

            // Fold/Unfold
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.tree.toggle();
            }

            _ => (),
        }
    }

    fn postrun(&mut self) {
        self.tree.position = 0;
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
