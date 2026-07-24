use promkit_core::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
};

use super::jsonz::{JsonNode, Row};
use crate::structured::{ContainerNode, ContainerType};

/// Defines the behavior for handling lines that
/// exceed the available width in the terminal when rendering JSON data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowMode {
    #[default]
    /// Truncates lines that exceed the available width
    /// and appends an ellipsis character (…).
    Truncate,
    /// Wraps lines that exceed the available width
    /// onto the next line without truncation.
    Wrap,
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
    /// Formats a Vec<Row> into Vec<StyledGraphemes> with appropriate styling and width limits
    pub fn render_terminal_rows(&self, rows: &[Row], width: u16) -> Vec<StyledGraphemes> {
        let mut formatted = Vec::new();
        let width = width as usize;

        for (i, row) in rows.iter().enumerate() {
            let indent = StyledGraphemes::from(" ".repeat(self.indent * row.depth));
            let mut parts = Vec::new();

            if let Some(key) = &row.key {
                parts.push(
                    StyledGraphemes::from(format!("\"{}\"", key)).apply_style(self.key_style),
                );
                parts.push(StyledGraphemes::from(": "));
            }

            match &row.node {
                JsonNode::Null => {
                    parts.push(StyledGraphemes::from("null").apply_style(self.null_value_style));
                }
                JsonNode::Boolean(b) => {
                    parts.push(
                        StyledGraphemes::from(b.to_string()).apply_style(self.boolean_value_style),
                    );
                }
                JsonNode::Number(n) => {
                    parts.push(
                        StyledGraphemes::from(n.to_string()).apply_style(self.number_value_style),
                    );
                }
                JsonNode::String(s) => {
                    let escaped = s.replace('\n', "\\n");
                    parts.push(
                        StyledGraphemes::from(format!("\"{}\"", escaped))
                            .apply_style(self.string_value_style),
                    );
                }
                JsonNode::Container(node) => match node {
                    ContainerNode::Empty { typ } => {
                        let bracket_style = match typ {
                            ContainerType::Object => self.curly_brackets_style,
                            ContainerType::Array => self.square_brackets_style,
                        };
                        parts.push(
                            StyledGraphemes::from(typ.empty_str()).apply_style(bracket_style),
                        );
                    }
                    ContainerNode::Open { typ, collapsed, .. } => {
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
                            parts.push(
                                StyledGraphemes::from(typ.open_str()).apply_style(bracket_style),
                            );
                        }
                    }
                    ContainerNode::Close { typ, .. } => {
                        let bracket_style = match typ {
                            ContainerType::Object => self.curly_brackets_style,
                            ContainerType::Array => self.square_brackets_style,
                        };
                        // We don't need to check collapsed here because:
                        // 1. If the corresponding Open is collapsed, this Close will be skipped during `extract_rows`
                        // 2. If the Open is not collapsed, we want to show the closing bracket
                        parts.push(
                            StyledGraphemes::from(typ.close_str()).apply_style(bracket_style),
                        );
                    }
                },
            }

            if i + 1 < rows.len() {
                if matches!(
                    &rows[i + 1].node,
                    JsonNode::Container(ContainerNode::Close { .. })
                ) {
                } else if matches!(
                    &rows[i].node,
                    JsonNode::Container(ContainerNode::Open {
                        collapsed: false,
                        ..
                    })
                ) {
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
                OverflowMode::Truncate => {
                    line = line.truncated_line_with_ellipsis(width, &StyledGraphemes::from("…"));
                    formatted.push(line);
                }
                OverflowMode::Wrap => {
                    formatted.extend(line.wrapped_lines(width));
                }
            }
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    mod render_terminal_rows {
        use super::*;

        use crate::structured::json::jsonz::create_rows;

        #[test]
        fn test_ellipsis_mode_truncates_with_ellipsis() {
            let value = json!({
                "very_long_key": "abcdefghijklmnopqrstuvwxyz",
            });
            let rows = create_rows([&value]);
            let width = 12;

            let lines = Config {
                indent: 2,
                overflow_mode: OverflowMode::Truncate,
                ..Default::default()
            }
            .render_terminal_rows(&rows, width);

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
                overflow_mode: OverflowMode::Wrap,
                ..Default::default()
            }
            .render_terminal_rows(&rows, width);

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
            assert_eq!(formatter.overflow_mode, OverflowMode::Truncate);
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
                overflow_mode = "Wrap"
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
            assert_eq!(formatter.overflow_mode, OverflowMode::Wrap);
        }
    }
}
