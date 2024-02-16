use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::ContentStyle,
    },
    grapheme::{trim, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable},
    tree::{NodeWithDepth, Tree},
};

#[derive(Clone)]
pub struct Renderer {
    pub tree: Tree,

    /// Symbol representing folded items.
    pub folded_symbol: String,
    /// Symbol representing unfolded items.
    pub unfolded_symbol: String,

    /// Style for selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected line.
    pub inactive_item_style: ContentStyle,

    /// Window size.
    pub screen_lines: Option<usize>,
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let symbol = |item: &NodeWithDepth| -> &str {
            if item.is_leaf || !item.children_visible {
                &self.folded_symbol
            } else {
                &self.unfolded_symbol
            }
        };

        let matrix = self
            .tree
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.tree.position() {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}{}",
                            symbol(item),
                            " ".repeat(item.data_from_root.len() + 1),
                            item.data
                        ),
                        self.active_item_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}{}",
                            " ".repeat(Graphemes::new(symbol(item)).widths()),
                            " ".repeat(item.data_from_root.len() + 1),
                            item.data
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        Pane::new(trimed, self.tree.position(), self.screen_lines)
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
        self.tree.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
