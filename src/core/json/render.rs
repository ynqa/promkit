use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::{Attribute, ContentStyle},
    },
    grapheme::{trim, StyledGraphemes},
    pane::Pane,
    AsAny, Error, EventAction, Result,
};

use super::{JsonSyntaxKind, JsonTree};

/// Represents a renderer for the `Json` component,
/// capable of visualizing JSON structures in a pane.
/// It supports interactive exploration of the JSON structure, including folding and unfolding
/// of JSON elements. The renderer highlights the currently selected line and can be configured
/// to render a specific number of lines with a specified indentation level for nested elements.
#[derive(Clone)]
pub struct Renderer {
    pub json: JsonTree,

    /// Style for {}.
    pub curly_brackets_style: ContentStyle,
    /// Style for [].
    pub square_brackets_style: ContentStyle,
    /// Style for "key".
    pub key_style: ContentStyle,
    /// Style for string values.
    pub string_value_style: ContentStyle,
    /// Style for number values.
    pub number_value_style: ContentStyle,
    /// Style for boolean values.
    pub boolean_value_style: ContentStyle,
    /// Style for null values.
    pub null_value_style: ContentStyle,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,

    /// The number of spaces used for indentation in the rendered JSON structure.
    /// This value multiplies with the indentation level of a JSON element to determine
    /// the total indentation space. For example, an `indent` value of 4 means each
    /// indentation level will be 4 spaces wide.
    pub indent: usize,
}

impl crate::Renderer for Renderer {
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
                | JsonSyntaxKind::ArrayEntry { indent, .. } => *indent * self.indent,
            }
        };

        let value = |v: &serde_json::Value| -> StyledGraphemes {
            match v {
                serde_json::Value::String(s) => {
                    StyledGraphemes::from_str(format!("\"{}\"", s), self.string_value_style)
                }
                serde_json::Value::Number(n) => {
                    StyledGraphemes::from_str(n.to_string(), self.number_value_style)
                }
                serde_json::Value::Bool(b) => {
                    StyledGraphemes::from_str(b.to_string(), self.boolean_value_style)
                }
                serde_json::Value::Null => StyledGraphemes::from_str("null", self.null_value_style),
                _ => StyledGraphemes::from(""),
            }
        };

        let syntax = |kind: &JsonSyntaxKind| -> StyledGraphemes {
            match kind {
                JsonSyntaxKind::MapStart { key, .. } => match key {
                    Some(key) => StyledGraphemes::from_iter([
                        StyledGraphemes::from_str(format!("\"{}\"", key), self.key_style),
                        StyledGraphemes::from(": "),
                        StyledGraphemes::from_str("{", self.curly_brackets_style),
                    ]),
                    None => StyledGraphemes::from_str("{", self.curly_brackets_style),
                },
                JsonSyntaxKind::MapEnd { is_last, .. } => {
                    if *is_last {
                        StyledGraphemes::from_str("}", self.curly_brackets_style)
                    } else {
                        StyledGraphemes::from_iter([
                            StyledGraphemes::from_str("}", self.curly_brackets_style),
                            StyledGraphemes::from(","),
                        ])
                    }
                }
                JsonSyntaxKind::MapFolded { key, is_last, .. } => {
                    let token = match key {
                        Some(key) => StyledGraphemes::from_iter([
                            StyledGraphemes::from_str(format!("\"{}\"", key), self.key_style),
                            StyledGraphemes::from(": "),
                            StyledGraphemes::from_str("{...}", self.curly_brackets_style),
                        ]),
                        None => StyledGraphemes::from_str("{...}", self.curly_brackets_style),
                    };
                    if *is_last {
                        token
                    } else {
                        StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                    }
                }
                JsonSyntaxKind::MapEntry { kv, is_last, .. } => {
                    let token = StyledGraphemes::from_iter([
                        StyledGraphemes::from_str(format!("\"{}\"", kv.0), self.key_style),
                        StyledGraphemes::from(": "),
                        value(&kv.1),
                    ]);
                    if *is_last {
                        token
                    } else {
                        StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                    }
                }
                JsonSyntaxKind::ArrayStart { key, .. } => match key {
                    Some(key) => StyledGraphemes::from_iter([
                        StyledGraphemes::from_str(format!("\"{}\"", key), self.key_style),
                        StyledGraphemes::from(": "),
                        StyledGraphemes::from_str("[", self.square_brackets_style),
                    ]),
                    None => StyledGraphemes::from_str("[", self.square_brackets_style),
                },
                JsonSyntaxKind::ArrayEnd { is_last, .. } => {
                    if *is_last {
                        StyledGraphemes::from_str("]", self.square_brackets_style)
                    } else {
                        StyledGraphemes::from_iter([
                            StyledGraphemes::from_str("]", self.square_brackets_style),
                            StyledGraphemes::from(","),
                        ])
                    }
                }
                JsonSyntaxKind::ArrayFolded { key, is_last, .. } => {
                    let token = match key {
                        Some(key) => StyledGraphemes::from_iter([
                            StyledGraphemes::from_str(format!("\"{}\"", key), self.key_style),
                            StyledGraphemes::from(": "),
                            StyledGraphemes::from_str("[...]", self.square_brackets_style),
                        ]),
                        None => StyledGraphemes::from_str("[...]", self.square_brackets_style),
                    };
                    if *is_last {
                        token
                    } else {
                        StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                    }
                }
                JsonSyntaxKind::ArrayEntry { v, is_last, .. } => {
                    let token = StyledGraphemes::from_iter([value(v)]);
                    if *is_last {
                        token
                    } else {
                        StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                    }
                }
            }
        };

        let matrix = self
            .json
            .kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.json.position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(indent(kind))),
                        syntax(kind),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(indent(kind))),
                        syntax(kind),
                    ])
                    .apply_attribute_to_all(Attribute::Dim)
                }
            })
            .collect::<Vec<StyledGraphemes>>();

        let trimed = matrix.iter().map(|row| trim(width as usize, row)).collect();
        Pane::new(trimed, self.json.position(), self.lines)
    }

    /// Default key bindings for JSON.
    ///
    /// | Key                | Description
    /// | :--                | :--
    /// | <kbd> ↑ </kbd>     | Move the cursor backward
    /// | <kbd> ↓ </kbd>     | Move the cursor forward
    /// | <kbd> Space </kbd> | Switch fold/unfold at the current node
    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Ok(EventAction::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Err(Error::Interrupted("ctrl+c".into())),

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
        Ok(EventAction::Continue)
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
