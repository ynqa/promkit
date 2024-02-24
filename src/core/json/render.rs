use std::any::Any;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::{Color, ContentStyle},
    },
    grapheme::{trim, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable},
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

    /// Style for the selected line.
    pub active_item_background_color: Color,
    /// Style for un-selected lines.
    pub inactive_item_background_color: Color,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,

    /// The number of spaces used for indentation in the rendered JSON structure.
    /// This value multiplies with the indentation level of a JSON element to determine
    /// the total indentation space. For example, an `indent` value of 4 means each
    /// indentation level will be 4 spaces wide.
    pub indent: usize,
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
                | JsonSyntaxKind::ArrayEntry { indent, .. } => *indent * self.indent,
            }
        };

        let apply_bgc_by_active = |style: ContentStyle, active: bool| -> ContentStyle {
            let mut ret = style.clone();
            if active {
                ret.background_color = Some(self.active_item_background_color);
            } else {
                ret.background_color = Some(self.inactive_item_background_color);
            }
            ret
        };

        let syntax = |kind: &JsonSyntaxKind, active: bool| -> Graphemes {
            match kind {
                JsonSyntaxKind::MapStart { key, .. } => match key {
                    Some(key) => Graphemes::from_iter([
                        Graphemes::new_with_style(
                            format!("\"{}\"", key),
                            apply_bgc_by_active(self.key_style, active),
                        ),
                        Graphemes::from(": "),
                        Graphemes::new_with_style(
                            "{",
                            apply_bgc_by_active(self.curly_brackets_style, active),
                        ),
                    ]),
                    None => Graphemes::new_with_style(
                        "{",
                        apply_bgc_by_active(self.curly_brackets_style, active),
                    ),
                },
                JsonSyntaxKind::MapEnd { is_last, .. } => {
                    if *is_last {
                        Graphemes::new_with_style(
                            "}",
                            apply_bgc_by_active(self.curly_brackets_style, active),
                        )
                    } else {
                        Graphemes::from_iter([
                            Graphemes::new_with_style(
                                "}",
                                apply_bgc_by_active(self.curly_brackets_style, active),
                            ),
                            Graphemes::from(","),
                        ])
                    }
                }
                JsonSyntaxKind::MapFolded { key, is_last, .. } => {
                    let mut token = match key {
                        Some(key) => Graphemes::from_iter([
                            Graphemes::new_with_style(
                                format!("\"{}\"", key),
                                apply_bgc_by_active(self.key_style, active),
                            ),
                            Graphemes::from(": "),
                            Graphemes::new_with_style(
                                "{...}",
                                apply_bgc_by_active(self.curly_brackets_style, active),
                            ),
                        ]),
                        None => Graphemes::new_with_style(
                            "{...}",
                            apply_bgc_by_active(self.curly_brackets_style, active),
                        ),
                    };
                    if *is_last {
                        token
                    } else {
                        Graphemes::from_iter([token, Graphemes::from(",")])
                    }
                }
                // TODO: fix below
                JsonSyntaxKind::MapEntry { kv, is_last, .. } => {
                    let mut token = format!("\"{}\": {}", kv.0, kv.1);
                    if !*is_last {
                        token.push(',');
                    }
                    token
                }
                JsonSyntaxKind::ArrayStart { key, .. } => match key {
                    Some(key) => format!("\"{}\": [", key),
                    None => "[".to_string(),
                },
                JsonSyntaxKind::ArrayEnd { is_last, .. } => {
                    let mut token = "]".to_string();
                    if !*is_last {
                        token.push(',');
                    }
                    token
                }
                JsonSyntaxKind::ArrayFolded { key, is_last, .. } => {
                    let mut token = match key {
                        Some(key) => format!("\"{}\": [...]", key),
                        None => "[...]".to_string(),
                    };
                    if !*is_last {
                        token.push(',');
                    }
                    token
                }
                JsonSyntaxKind::ArrayEntry { v, is_last, .. } => {
                    let mut token = format!("{}", v);
                    if !*is_last {
                        token.push(',');
                    }
                    token
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
                    Graphemes::from_iter([
                        Graphemes::from(" ".repeat(indent(kind))),
                        syntax(kind, true),
                    ])
                } else {
                    Graphemes::from_iter([
                        Graphemes::from(" ".repeat(indent(kind))),
                        syntax(kind, false),
                    ])
                }
            })
            .collect::<Vec<Graphemes>>();

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
