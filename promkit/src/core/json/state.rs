use crate::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::{trim, StyledGraphemes},
    impl_as_any,
    pane::Pane,
    PaneFactory,
};

use super::{JsonStream, JsonSyntaxKind};

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

/// Represents the state of a JSON stream within the application.
///
/// This struct holds the current JSON stream being processed and provides
/// methods to interact with and manipulate the stream according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    pub stream: JsonStream,

    pub theme: Theme,
}

impl_as_any!(State);

impl State {
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

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.theme.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let viewport = self.stream.viewport_range(height);

        let layout = self
            .stream
            .flatten_kinds()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= viewport.0 && *i < viewport.1)
            .map(|(i, kind)| {
                if i == self.stream.cursor.cross_contents_position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::State::indent_level(kind, &self.theme)),
                        ),
                        super::State::gen_syntax_style(kind, &self.theme)
                            .apply_attribute_to_all(self.theme.active_item_attribute),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::State::indent_level(kind, &self.theme)),
                        ),
                        super::State::gen_syntax_style(kind, &self.theme),
                    ])
                    .apply_attribute_to_all(self.theme.inactive_item_attribute)
                }
            })
            .map(|row| trim(width as usize, &row))
            .collect::<Vec<StyledGraphemes>>();

        Pane::new(layout)
    }
}
