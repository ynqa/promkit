use crate::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
};

use super::{ContainerType, Row, Value};

#[derive(Clone)]
pub struct RowFormatter {
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

    /// The number of spaces used for indentation in the rendered JSON structure.
    /// This value multiplies with the indentation level of a JSON element to determine
    /// the total indentation space. For example, an `indent` value of 4 means each
    /// indentation level will be 4 spaces wide.
    pub indent: usize,
}

impl Default for RowFormatter {
    fn default() -> Self {
        Self {
            curly_brackets_style: Default::default(),
            square_brackets_style: Default::default(),
            key_style: Default::default(),
            string_value_style: Default::default(),
            number_value_style: Default::default(),
            boolean_value_style: Default::default(),
            null_value_style: Default::default(),
            active_item_attribute: Attribute::NoBold,
            inactive_item_attribute: Attribute::NoBold,
            indent: Default::default(),
        }
    }
}

impl RowFormatter {
    /// Formats a Vec<Row> into Vec<StyledGraphemes> with appropriate styling and width limits
    pub fn format_for_terminal_display(&self, rows: &[Row], width: u16) -> Vec<StyledGraphemes> {
        let mut formatted = Vec::new();
        let width = width as usize;

        for (i, row) in rows.iter().enumerate() {
            let indent = StyledGraphemes::from(" ".repeat(self.indent * row.depth));
            let mut parts = Vec::new();

            if let Some(key) = &row.k {
                parts.push(
                    StyledGraphemes::from(format!("\"{}\"", key)).apply_style(self.key_style),
                );
                parts.push(StyledGraphemes::from(": "));
            }

            match &row.v {
                Value::Null => {
                    parts.push(StyledGraphemes::from("null").apply_style(self.null_value_style));
                }
                Value::Boolean(b) => {
                    parts.push(
                        StyledGraphemes::from(b.to_string()).apply_style(self.boolean_value_style),
                    );
                }
                Value::Number(n) => {
                    parts.push(
                        StyledGraphemes::from(n.to_string()).apply_style(self.number_value_style),
                    );
                }
                Value::String(s) => {
                    let escaped = s.replace('\n', "\\n");
                    parts.push(
                        StyledGraphemes::from(format!("\"{}\"", escaped))
                            .apply_style(self.string_value_style),
                    );
                }
                Value::Empty { typ } => {
                    let bracket_style = match typ {
                        ContainerType::Object => self.curly_brackets_style,
                        ContainerType::Array => self.square_brackets_style,
                    };
                    parts.push(StyledGraphemes::from(typ.empty_str()).apply_style(bracket_style));
                }
                Value::Open { typ, collapsed, .. } => {
                    let bracket_style = match typ {
                        ContainerType::Object => self.curly_brackets_style,
                        ContainerType::Array => self.square_brackets_style,
                    };
                    if *collapsed {
                        parts.push(
                            StyledGraphemes::from(typ.collapsed_preview())
                                .apply_style(bracket_style),
                        );
                    } else {
                        parts
                            .push(StyledGraphemes::from(typ.open_str()).apply_style(bracket_style));
                    }
                }
                Value::Close { typ, .. } => {
                    let bracket_style = match typ {
                        ContainerType::Object => self.curly_brackets_style,
                        ContainerType::Array => self.square_brackets_style,
                    };
                    // We don't need to check collapsed here because:
                    // 1. If the corresponding Open is collapsed, this Close will be skipped during `extract_rows`
                    // 2. If the Open is not collapsed, we want to show the closing bracket
                    parts.push(StyledGraphemes::from(typ.close_str()).apply_style(bracket_style));
                }
            }

            if i + 1 < rows.len() {
                if let Value::Close { .. } = rows[i + 1].v {
                } else if let Value::Open {
                    collapsed: false, ..
                } = rows[i].v
                {
                } else {
                    parts.push(StyledGraphemes::from(","));
                }
            }

            let mut content: StyledGraphemes = parts.into_iter().collect();

            // Note that `extract_rows_from_current`
            // returns rows starting from the current position,
            // so the first row should always be highlighted as active
            content = content.apply_attribute(if i == 0 {
                self.active_item_attribute
            } else {
                self.inactive_item_attribute
            });

            let mut line: StyledGraphemes = vec![indent, content].into_iter().collect();

            if line.widths() > width {
                let ellipsis: StyledGraphemes = StyledGraphemes::from("…");
                let mut truncated = StyledGraphemes::default();
                let mut current_width = 0;
                for g in line.iter() {
                    if current_width + g.width() + ellipsis.widths() > width {
                        break;
                    }
                    truncated.push_back(g.clone());
                    current_width += g.width();
                }
                line = vec![truncated, ellipsis].into_iter().collect();
            }

            formatted.push(line);
        }

        formatted
    }

    /// Formats a slice of Rows to a raw JSON string, ignoring collapsed and truncated states
    pub fn format_raw_json(&self, rows: &[Row]) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    mod format_raw_json {
        use std::str::FromStr;

        use crate::jsonz::{create_rows, format};

        #[test]
        fn test() {
            let expected = r#"
{
    "array": [
        {
            "key": "value"
        },
        [
            1,
            2,
            3
        ],
        {
            "nested": true
        }
    ],
    "object": {
        "array": [
            1,
            2,
            3
        ],
        "nested": {
            "value": "test"
        }
    }
}"#
            .trim();

            assert_eq!(
                format::RowFormatter {
                    indent: 4,
                    ..Default::default()
                }
                .format_raw_json(&create_rows([
                    &serde_json::Value::from_str(&expected).unwrap()
                ])),
                expected,
            );
        }
    }
}
