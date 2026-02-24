use promkit_core::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
};

use super::jsonz::{ContainerType, Row, Value};

/// Defines the behavior for handling lines that
/// exceed the available width in the terminal when rendering JSON data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowMode {
    #[default]
    /// Truncates lines that exceed the available width
    /// and appends an ellipsis character (…).
    Ellipsis,
    /// Wraps lines that exceed the available width
    /// onto the next line without truncation.
    LineWrap,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone)]
pub struct Config {
    /// Style for {}.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub curly_brackets_style: ContentStyle,
    /// Style for [].
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub square_brackets_style: ContentStyle,
    /// Style for "key".
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub key_style: ContentStyle,
    /// Style for string values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub string_value_style: ContentStyle,
    /// Style for number values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub number_value_style: ContentStyle,
    /// Style for boolean values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub boolean_value_style: ContentStyle,
    /// Style for null values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub null_value_style: ContentStyle,

    /// Attribute for the selected line.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::attribute_serde")
    )]
    pub active_item_attribute: Attribute,
    /// Attribute for unselected lines.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::attribute_serde")
    )]
    pub inactive_item_attribute: Attribute,

    /// The number of spaces used for indentation in the rendered JSON structure.
    /// This value multiplies with the indentation level of a JSON element to determine
    /// the total indentation space. For example, an `indent` value of 4 means each
    /// indentation level will be 4 spaces wide.
    pub indent: usize,

    /// Rendering behavior when a line exceeds the terminal width.
    pub overflow_mode: OverflowMode,
    /// Number of lines available for rendering.
    pub lines: Option<usize>,
}

impl Default for Config {
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
            overflow_mode: OverflowMode::default(),
            lines: Default::default(),
        }
    }
}

impl Config {
    fn truncate_line_with_ellipsis(line: StyledGraphemes, width: usize) -> StyledGraphemes {
        if line.widths() <= width {
            return line;
        }

        if width == 0 {
            return StyledGraphemes::default();
        }

        let ellipsis: StyledGraphemes = StyledGraphemes::from("…");
        let ellipsis_width = ellipsis.widths();
        if width <= ellipsis_width {
            return ellipsis;
        }

        let mut truncated = StyledGraphemes::default();
        let mut current_width = 0;
        for g in line.iter() {
            if current_width + g.width() + ellipsis_width > width {
                break;
            }
            truncated.push_back(g.clone());
            current_width += g.width();
        }

        vec![truncated, ellipsis].into_iter().collect()
    }

    fn wrap_line(line: StyledGraphemes, width: usize) -> Vec<StyledGraphemes> {
        let mut wrapped = vec![StyledGraphemes::default()];
        let mut current_width = 0;

        for g in line.iter() {
            if g.width() > width {
                continue;
            }
            if current_width + g.width() > width {
                wrapped.push(StyledGraphemes::default());
                current_width = 0;
            }
            wrapped
                .last_mut()
                .expect("wrapped always contains at least one row")
                .push_back(g.clone());
            current_width += g.width();
        }

        wrapped
    }

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

            match self.overflow_mode {
                OverflowMode::Ellipsis => {
                    line = Self::truncate_line_with_ellipsis(line, width);
                    formatted.push(line);
                }
                OverflowMode::LineWrap => {
                    formatted.extend(Self::wrap_line(line, width));
                }
            }
        }

        formatted
    }

    /// Formats a slice of Rows to a raw JSON string, ignoring collapsed and truncated states
    pub fn format_raw_json(&self, rows: &[Row]) -> String {
        let mut result = String::new();
        let mut first_in_container = true;

        for (i, row) in rows.iter().enumerate() {
            // Add indentation
            if !matches!(row.v, Value::Close { .. }) {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&" ".repeat(self.indent * row.depth));
            }

            // Add key if present
            if let Some(key) = &row.k {
                result.push('"');
                result.push_str(key);
                result.push_str("\": ");
            }

            // Add value
            match &row.v {
                Value::Null => result.push_str("null"),
                Value::Boolean(b) => result.push_str(&b.to_string()),
                Value::Number(n) => result.push_str(&n.to_string()),
                Value::String(s) => {
                    result.push('"');
                    result.push_str(&s.replace('\n', "\\n"));
                    result.push('"');
                }
                Value::Empty { typ } => {
                    result.push_str(match typ {
                        ContainerType::Object => "{}",
                        ContainerType::Array => "[]",
                    });
                }
                Value::Open { typ, .. } => {
                    result.push(match typ {
                        ContainerType::Object => '{',
                        ContainerType::Array => '[',
                    });
                }
                Value::Close { typ, .. } => {
                    if !first_in_container {
                        result.push('\n');
                        result.push_str(&" ".repeat(self.indent * row.depth));
                    }
                    result.push(match typ {
                        ContainerType::Object => '}',
                        ContainerType::Array => ']',
                    });
                }
            }

            // Add comma if needed
            if i + 1 < rows.len() {
                if let Value::Close { .. } = rows[i + 1].v {
                    // Don't add comma before closing bracket
                } else if let Value::Open { .. } = rows[i].v {
                    // Don't add comma after opening bracket
                } else {
                    result.push(',');
                }
            }

            if let Value::Open { .. } = row.v {
                first_in_container = true;
            } else {
                first_in_container = false;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    mod format_raw_json {
        use std::str::FromStr;

        use super::*;

        use crate::jsonstream::jsonz::create_rows;

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
                Config {
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

    mod format_for_terminal_display {
        use super::*;

        use crate::jsonstream::jsonz::create_rows;

        #[test]
        fn test_ellipsis_mode_truncates_with_ellipsis() {
            let value = json!({
                "very_long_key": "abcdefghijklmnopqrstuvwxyz",
            });
            let rows = create_rows([&value]);
            let width = 12;

            let lines = Config {
                indent: 2,
                overflow_mode: OverflowMode::Ellipsis,
                ..Default::default()
            }
            .format_for_terminal_display(&rows, width);

            assert_eq!(lines.len(), rows.len());
            assert!(lines.iter().all(|line| line.widths() <= width as usize));
            assert!(
                lines
                    .iter()
                    .any(|line| line.chars().last().is_some_and(|ch| *ch == '…'))
            );
        }

        #[test]
        fn test_linewrap_mode_wraps_without_ellipsis() {
            let value = json!({
                "very_long_key": "abcdefghijklmnopqrstuvwxyz",
            });
            let rows = create_rows([&value]);
            let width = 12;

            let lines = Config {
                indent: 2,
                overflow_mode: OverflowMode::LineWrap,
                ..Default::default()
            }
            .format_for_terminal_display(&rows, width);

            assert!(lines.len() > rows.len());
            assert!(lines.iter().all(|line| line.widths() <= width as usize));
            assert!(
                lines
                    .iter()
                    .all(|line| !matches!(line.chars().last(), Some('…')))
            );
        }
    }

    #[cfg(feature = "serde")]
    mod serde_compatibility {
        use super::*;
        use promkit_core::crossterm::style::{Attributes, Color};

        #[test]
        fn missing_new_fields_are_filled_by_default() {
            let mut value = serde_json::to_value(Config {
                indent: 4,
                ..Default::default()
            })
            .unwrap();
            let obj = value.as_object_mut().unwrap();
            obj.remove("active_item_attribute");
            obj.remove("inactive_item_attribute");
            obj.remove("overflow_mode");
            obj.remove("lines");

            let formatter: Config = serde_json::from_value(value).unwrap();

            assert_eq!(formatter.indent, 4);
            assert_eq!(formatter.active_item_attribute, Attribute::NoBold);
            assert_eq!(formatter.inactive_item_attribute, Attribute::NoBold);
            assert_eq!(formatter.overflow_mode, OverflowMode::Ellipsis);
            assert_eq!(formatter.lines, None);
        }

        #[test]
        fn config_fields_are_fully_loaded_from_toml() {
            let input = r#"
indent = 4
lines = 7
curly_brackets_style = "attr=bold"
square_brackets_style = "attr=bold"
key_style = "fg=cyan"
string_value_style = "fg=green"
number_value_style = "fg=yellow"
boolean_value_style = "fg=magenta"
null_value_style = "fg=grey"
active_item_attribute = "underlined"
inactive_item_attribute = "dim"
overflow_mode = "LineWrap"
"#;

            let formatter: Config = toml::from_str(input).unwrap();

            assert_eq!(formatter.indent, 4);
            assert_eq!(formatter.lines, Some(7));
            assert_eq!(
                formatter.curly_brackets_style.attributes,
                Attributes::from(Attribute::Bold),
            );
            assert_eq!(
                formatter.square_brackets_style.attributes,
                Attributes::from(Attribute::Bold),
            );
            assert_eq!(formatter.key_style.foreground_color, Some(Color::Cyan));
            assert_eq!(
                formatter.string_value_style.foreground_color,
                Some(Color::Green),
            );
            assert_eq!(
                formatter.number_value_style.foreground_color,
                Some(Color::Yellow)
            );
            assert_eq!(
                formatter.boolean_value_style.foreground_color,
                Some(Color::Magenta),
            );
            assert_eq!(
                formatter.null_value_style.foreground_color,
                Some(Color::Grey)
            );
            assert_eq!(formatter.active_item_attribute, Attribute::Underlined);
            assert_eq!(formatter.inactive_item_attribute, Attribute::Dim);
            assert_eq!(formatter.overflow_mode, OverflowMode::LineWrap);
        }
    }
}

pub type Formatter = Config;
pub type RowFormatter = Config;
