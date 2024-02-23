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

use super::{Json, JsonSyntaxKind};

#[derive(Clone)]
pub struct Renderer {
    pub json: Json,

    /// Style for the selected line.
    pub active_item_style: ContentStyle,
    /// Style for un-selected lines.
    pub inactive_item_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let indent = |kind: &JsonSyntaxKind| -> usize {
            match kind {
                JsonSyntaxKind::MapStart { indent, .. }
                | JsonSyntaxKind::MapEnd { indent, .. }
                | JsonSyntaxKind::MapFolded { indent, .. }
                | JsonSyntaxKind::MapEntry { indent, .. }
                | JsonSyntaxKind::ArrayFolded { indent, .. }
                | JsonSyntaxKind::ArrayStart { indent, .. }
                | JsonSyntaxKind::ArrayEnd { indent, .. }
                | JsonSyntaxKind::ArrayEntry { indent, .. } => *indent,
            }
        };

        let syntax = |kind: &JsonSyntaxKind| -> String {
            match kind {
                JsonSyntaxKind::MapStart { key, .. } => match key {
                    Some(key) => format!("\"{}\": {{", key),
                    None => "{".to_string(),
                },
                JsonSyntaxKind::MapEnd { is_last, .. } => {
                    let mut token = "}".to_string();
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
                JsonSyntaxKind::MapFolded { key, is_last, .. } => {
                    let mut token = match key {
                        Some(key) => format!("\"{}\": {{...}}", key),
                        None => "{...}".to_string(),
                    };
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
                JsonSyntaxKind::MapEntry { kv, is_last, .. } => {
                    let mut token = format!("\"{}\": {}", kv.0, kv.1);
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
                JsonSyntaxKind::ArrayStart { key, .. } => match key {
                    Some(key) => format!("\"{}\": [", key),
                    None => "[".to_string(),
                },
                JsonSyntaxKind::ArrayEnd { is_last, .. } => {
                    let mut token = "]".to_string();
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
                JsonSyntaxKind::ArrayFolded { key, is_last, .. } => {
                    let mut token = match key {
                        Some(key) => format!("\"{}\": [...]", key),
                        None => "[...]".to_string(),
                    };
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
                JsonSyntaxKind::ArrayEntry { v, is_last, .. } => {
                    let mut token = format!("{}", v);
                    if !*is_last {
                        token.push_str(",");
                    }
                    token
                },
            }
        };

        let matrix = self
            .json
            .kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.json.position() {
                    Graphemes::new_with_style(
                        format!("{}{}", " ".repeat(indent(kind)), syntax(kind)),
                        self.active_item_style,
                    )
                } else {
                    Graphemes::new_with_style(
                        format!("{}{}", " ".repeat(indent(kind)), syntax(kind)),
                        self.inactive_item_style,
                    )
                }
            })
            .collect::<Vec<Graphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        Pane::new(trimed, self.json.position(), self.lines)
    }

    /// Default key bindings for tree.
    ///
    /// | Key                | Description
    /// | :--                | :--
    /// | <kbd> ↑ </kbd>     | Move the cursor backward
    /// | <kbd> ↓ </kbd>     | Move the cursor forward
    /// | <kbd> Space </kbd> | Switch fold/unfold at the current node
    fn handle_event(&mut self, event: &Event) {
        match event {
            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.json.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.json.forward();
            }

            // Fold/Unfold
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.json.toggle();
            }

            _ => (),
        }
    }

    fn postrun(&mut self) {
        self.json.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
