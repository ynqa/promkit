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
    tree::{NodeWithDepth, Tree},
};

#[derive(Clone)]
pub struct Renderer {
    pub tree: Tree,

    /// Style for selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected line.
    pub inactive_item_style: ContentStyle,

    /// Symbol representing folded items.
    pub folded_symbol: String,
    /// Symbol representing unfolded items.
    pub unfolded_symbol: String,

    /// Window size.
    pub window_size: Option<usize>,
}

impl State<Renderer> {
    pub fn try_new(
        tree: Tree,
        active_item_style: ContentStyle,
        inactive_item_style: ContentStyle,
        folded_symbol: String,
        unfolded_symbol: String,
        window_size: Option<usize>,
    ) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(Renderer {
            tree,
            active_item_style,
            inactive_item_style,
            folded_symbol,
            unfolded_symbol,
            window_size,
        })))
    }
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> crate::pane::Pane {
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
                        format!("{}{}{}", symbol(item), " ".repeat(item.depth), item.data),
                        self.active_item_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!(
                            "{}{}{}",
                            " ".repeat(Graphemes::new(symbol(item)).widths()),
                            " ".repeat(item.depth),
                            item.data
                        ),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        Pane::new(trimed, self.tree.position(), self.window_size)
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
