use crate::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
    pane::Pane,
    PaneFactory,
};

use super::{JsonStream, JsonSyntaxKind};

/// Represents the state of a JSON stream within the application.
///
/// This struct holds the current JSON stream being processed and provides
/// methods to interact with and manipulate the stream according to the
/// application's needs. It also contains a theme configuration for styling
/// the JSON output.
#[derive(Clone)]
pub struct State {
    pub stream: JsonStream,

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

impl State {
    pub fn indent_level(&self, kind: &JsonSyntaxKind) -> usize {
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
    }

    fn format_value(&self, v: &serde_json::Value) -> StyledGraphemes {
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
    }

    pub fn gen_syntax_style(&self, kind: &JsonSyntaxKind) -> StyledGraphemes {
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
                    self.format_value(&kv.1),
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
                let token = StyledGraphemes::from_iter([self.format_value(v)]);
                if *is_last {
                    token
                } else {
                    StyledGraphemes::from_iter([token, StyledGraphemes::from(",")])
                }
            }
        }
    }

    fn styled_json(&self) -> Vec<StyledGraphemes> {
        self.stream
            .flatten_kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.stream.cursor.cross_contents_position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(self.indent_level(kind))),
                        self.gen_syntax_style(kind)
                            .apply_attribute(self.active_item_attribute),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(" ".repeat(self.indent_level(kind))),
                        self.gen_syntax_style(kind),
                    ])
                    .apply_attribute(self.inactive_item_attribute)
                }
            })
            .collect()
    }

    pub fn json_str(&self) -> String {
        self.styled_json()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let styled_json = self.styled_json();
        let matrix = styled_json
            .into_iter()
            .enumerate()
            .filter(|(i, _)| {
                *i >= self.stream.cursor.cross_contents_position()
                    && *i < self.stream.cursor.cross_contents_position() + height
            })
            .fold((vec![], 0), |(mut acc, pos), (_, item)| {
                let rows = item.matrixify(width as usize, height, 0).0;
                if pos < self.stream.cursor.cross_contents_position() + height {
                    acc.extend(rows);
                }
                (acc, pos + 1)
            });

        Pane::new(matrix.0, 0)
    }
}
