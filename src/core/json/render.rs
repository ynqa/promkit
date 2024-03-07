use std::any::Any;

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, ContentStyle},
    },
    grapheme::{trim, StyledGraphemes},
    keymap::KeymapManager,
    pane::Pane,
    AsAny, EventAction, Result,
};

use super::{Json, JsonSyntaxKind};

#[derive(Clone)]
pub struct Theme {
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

    /// Attribute for the selected line.
    pub active_item_attribute: Attribute,
    /// Attribute for unselected lines.
    pub inactive_item_attribute: Attribute,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,

    /// The number of spaces used for indentation in the rendered JSON structure.
    /// This value multiplies with the indentation level of a JSON element to determine
    /// the total indentation space. For example, an `indent` value of 4 means each
    /// indentation level will be 4 spaces wide.
    pub indent: usize,
}

/// Represents a renderer for the `Json` component,
/// capable of visualizing JSON structures in a pane.
/// It supports interactive exploration of the JSON structure, including folding and unfolding
/// of JSON elements. The renderer highlights the currently selected line and can be configured
/// to render a specific number of lines with a specified indentation level for nested elements.
#[derive(Clone)]
pub struct Renderer {
    pub json: Json,

    pub keymap: KeymapManager<Self>,

    pub theme: Theme,
}

impl Renderer {
    pub fn indent_level(kind: &JsonSyntaxKind, theme: &Theme) -> usize {
        match kind {
            JsonSyntaxKind::MapStart { indent, .. }
            | JsonSyntaxKind::MapEnd { indent, .. }
            | JsonSyntaxKind::MapFolded { indent, .. }
            | JsonSyntaxKind::MapEntry { indent, .. }
            | JsonSyntaxKind::ArrayFolded { indent, .. }
            | JsonSyntaxKind::ArrayStart { indent, .. }
            | JsonSyntaxKind::ArrayEnd { indent, .. }
            | JsonSyntaxKind::ArrayEntry { indent, .. } => *indent * theme.indent,
        }
    }

    fn format_value(v: &serde_json::Value, theme: &Theme) -> StyledGraphemes {
        match v {
            serde_json::Value::String(s) => {
                StyledGraphemes::from_str(format!("\"{}\"", s), theme.string_value_style)
            }
            serde_json::Value::Number(n) => {
                StyledGraphemes::from_str(n.to_string(), theme.number_value_style)
            }
            serde_json::Value::Bool(b) => {
                StyledGraphemes::from_str(b.to_string(), theme.boolean_value_style)
            }
            serde_json::Value::Null => StyledGraphemes::from_str("null", theme.null_value_style),
            _ => StyledGraphemes::from(""),
        }
    }

    pub fn gen_syntax_style(kind: &JsonSyntaxKind, theme: &Theme) -> StyledGraphemes {
        match kind {
            JsonSyntaxKind::MapStart { key, .. } => match key {
                Some(key) => StyledGraphemes::from_iter([
                    StyledGraphemes::from_str(format!("\"{}\"", key), theme.key_style),
                    StyledGraphemes::from(": "),
                    StyledGraphemes::from_str("{", theme.curly_brackets_style),
                ]),
                None => StyledGraphemes::from_str("{", theme.curly_brackets_style),
            },
            JsonSyntaxKind::MapEnd { is_last, .. } => {
                if *is_last {
                    StyledGraphemes::from_str("}", theme.curly_brackets_style)
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from_str("}", theme.curly_brackets_style),
                        StyledGraphemes::from(","),
                    ])
                }
            }
            JsonSyntaxKind::MapFolded { key, is_last, .. } => {
                let token = match key {
                    Some(key) => StyledGraphemes::from_iter([
                        StyledGraphemes::from_str(format!("\"{}\"", key), theme.key_style),
                        StyledGraphemes::from(": "),
                        StyledGraphemes::from_str("{...}", theme.curly_brackets_style),
                    ]),
                    None => StyledGraphemes::from_str("{...}", theme.curly_brackets_style),
                };
                if *is_last {
                    token
                } else {
                    StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                }
            }
            JsonSyntaxKind::MapEntry { kv, is_last, .. } => {
                let token = StyledGraphemes::from_iter([
                    StyledGraphemes::from_str(format!("\"{}\"", kv.0), theme.key_style),
                    StyledGraphemes::from(": "),
                    Self::format_value(&kv.1, theme),
                ]);
                if *is_last {
                    token
                } else {
                    StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                }
            }
            JsonSyntaxKind::ArrayStart { key, .. } => match key {
                Some(key) => StyledGraphemes::from_iter([
                    StyledGraphemes::from_str(format!("\"{}\"", key), theme.key_style),
                    StyledGraphemes::from(": "),
                    StyledGraphemes::from_str("[", theme.square_brackets_style),
                ]),
                None => StyledGraphemes::from_str("[", theme.square_brackets_style),
            },
            JsonSyntaxKind::ArrayEnd { is_last, .. } => {
                if *is_last {
                    StyledGraphemes::from_str("]", theme.square_brackets_style)
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from_str("]", theme.square_brackets_style),
                        StyledGraphemes::from(","),
                    ])
                }
            }
            JsonSyntaxKind::ArrayFolded { key, is_last, .. } => {
                let token = match key {
                    Some(key) => StyledGraphemes::from_iter([
                        StyledGraphemes::from_str(format!("\"{}\"", key), theme.key_style),
                        StyledGraphemes::from(": "),
                        StyledGraphemes::from_str("[...]", theme.square_brackets_style),
                    ]),
                    None => StyledGraphemes::from_str("[...]", theme.square_brackets_style),
                };
                if *is_last {
                    token
                } else {
                    StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                }
            }
            JsonSyntaxKind::ArrayEntry { v, is_last, .. } => {
                let token = StyledGraphemes::from_iter([Self::format_value(v, theme)]);
                if *is_last {
                    token
                } else {
                    StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                }
            }
        }
    }
}

impl crate::Renderer for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let layout = self
            .json
            .kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.json.position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(Self::indent_level(kind, &self.theme))),
                        Self::gen_syntax_style(kind, &self.theme)
                            .apply_attribute_to_all(self.theme.active_item_attribute),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(Self::indent_level(kind, &self.theme))),
                        Self::gen_syntax_style(kind, &self.theme),
                    ])
                    .apply_attribute_to_all(self.theme.inactive_item_attribute)
                }
            })
            .map(|row| trim(width as usize, &row))
            .collect::<Vec<StyledGraphemes>>();

        Pane::new(layout, self.json.position(), self.theme.lines)
    }

    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        (self.keymap.get())(self, event)
    }

    fn postrun(&mut self) {
        self.json.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
