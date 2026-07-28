use promkit_core::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
};

use crate::structured::yaml::yamlz::{ContainerNode, ContainerType, Row, YamlNode};

/// Defines the behavior for handling lines that
/// exceed the available width in the terminal when rendering YAML data.
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
    /// Style for `{}` and `{...}`.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub map_style: ContentStyle,

    /// Style for `[]` and `[...]`.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub sequence_style: ContentStyle,

    /// Style for YAML keys.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub key_style: ContentStyle,

    /// Style for YAML tags, such as `!MyTag`.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub tag_style: ContentStyle,

    /// Style for string values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub string_style: ContentStyle,

    /// Style for number values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub number_style: ContentStyle,

    /// Style for boolean values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub boolean_style: ContentStyle,

    /// Style for null values.
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub null_style: ContentStyle,

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

    /// The number of spaces used for indentation in the rendered YAML structure.
    pub indent: usize,

    /// Rendering behavior when a line exceeds the terminal width.
    pub overflow_mode: OverflowMode,

    /// Number of lines available for rendering.
    pub lines: Option<usize>,
    /// Whether to display stable one-based line numbers to the left of the content.
    pub show_line_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map_style: Default::default(),
            sequence_style: Default::default(),
            key_style: Default::default(),
            tag_style: Default::default(),
            string_style: Default::default(),
            number_style: Default::default(),
            boolean_style: Default::default(),
            null_style: Default::default(),
            active_item_attribute: Attribute::NoBold,
            inactive_item_attribute: Attribute::NoBold,
            indent: 2,
            overflow_mode: OverflowMode::default(),
            lines: None,
            show_line_numbers: false,
        }
    }
}

impl Config {
    fn is_plain_yaml_string(s: &str) -> bool {
        if s.is_empty() || s.trim() != s {
            return false;
        }

        !s.contains([
            ':', '#', '{', '}', '[', ']', ',', '&', '*', '?', '|', '>', '!', '%', '@', '`', '\\',
            '"', '\'', '\n', '\r', '\t',
        ])
    }

    fn render_yaml_string(s: &str) -> String {
        if Self::is_plain_yaml_string(s) {
            s.to_string()
        } else {
            format!(
                "\"{}\"",
                s.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
            )
        }
    }

    fn render_collection_marker(&self, typ: &ContainerType) -> StyledGraphemes {
        let style = match typ {
            ContainerType::Object => self.map_style,
            ContainerType::Array => self.sequence_style,
        };
        StyledGraphemes::from(typ.empty_str()).apply_style(style)
    }

    fn render_collapsed_marker(&self, typ: &ContainerType) -> StyledGraphemes {
        let style = match typ {
            ContainerType::Object => self.map_style,
            ContainerType::Array => self.sequence_style,
        };
        StyledGraphemes::from(typ.collapsed_preview()).apply_style(style)
    }

    fn render_node(&self, node: &YamlNode) -> Option<StyledGraphemes> {
        match node {
            YamlNode::Tagged { tag, node } => {
                let tag_part =
                    StyledGraphemes::from(format!("{} ", tag)).apply_style(self.tag_style);
                match self.render_node(node) {
                    Some(value) => Some(vec![tag_part, value].into_iter().collect()),
                    None => Some(tag_part),
                }
            }
            YamlNode::Null => Some(StyledGraphemes::from("null").apply_style(self.null_style)),
            YamlNode::Boolean(b) => {
                Some(StyledGraphemes::from(b.to_string()).apply_style(self.boolean_style))
            }
            YamlNode::Number(n) => {
                Some(StyledGraphemes::from(n.to_string()).apply_style(self.number_style))
            }
            YamlNode::String(s) => Some(
                StyledGraphemes::from(Self::render_yaml_string(s)).apply_style(self.string_style),
            ),
            YamlNode::DocumentSeparator => Some(StyledGraphemes::from("---")),
            YamlNode::Container(node) => match node {
                ContainerNode::Empty { typ } => Some(self.render_collection_marker(typ)),
                ContainerNode::Open {
                    typ,
                    collapsed: true,
                    ..
                } => Some(self.render_collapsed_marker(typ)),
                ContainerNode::Open {
                    collapsed: false, ..
                } => None,
                ContainerNode::Close { .. } => None,
            },
        }
    }

    /// Format YAML rows into terminal lines with styling and width constraints.
    pub fn render_terminal_rows(&self, rows: &[Row], width: u16) -> Vec<StyledGraphemes> {
        self.render_rows(rows, 0, Some(width as usize))
    }

    /// Formats width-independent rows for the core renderer.
    pub fn render_content_rows(&self, rows: &[Row], active_row: usize) -> Vec<StyledGraphemes> {
        self.render_rows(rows, active_row, None)
    }

    fn render_rows(
        &self,
        rows: &[Row],
        active_row: usize,
        width: Option<usize>,
    ) -> Vec<StyledGraphemes> {
        let mut formatted = Vec::new();
        let mut i = 0;
        let mut rendered_row = 0;

        while i < rows.len() {
            let row = &rows[i];

            // Collection roots occupy depth 0 as an operation row, so their
            // children start at depth 1 internally. YAML does not render the
            // root collection itself; remove that structural depth from the
            // displayed indentation.
            let display_depth = row.depth.saturating_sub(1);
            let indent = StyledGraphemes::from(" ".repeat(self.indent * display_depth));
            let mut parts: Vec<StyledGraphemes> = Vec::new();

            if matches!(row.node, YamlNode::DocumentSeparator) {
                parts.push(StyledGraphemes::from("---"));
            } else {
                if row.is_sequence_item {
                    parts.push(StyledGraphemes::from("- "));
                }

                if let Some(key) = &row.key {
                    parts.push(
                        StyledGraphemes::from(Self::render_yaml_string(key))
                            .apply_style(self.key_style),
                    );
                    parts.push(StyledGraphemes::from(": "));
                }

                if let Some(value) = self.render_node(&row.node) {
                    parts.push(value);
                }

                if matches!(
                    row.node,
                    YamlNode::Container(ContainerNode::Open {
                        typ: ContainerType::Object,
                        collapsed: false,
                        ..
                    })
                ) && row.is_sequence_item
                    && let Some(next_row) = rows.get(i + 1)
                    && next_row.depth == row.depth + 1
                    && !next_row.is_sequence_item
                    && let Some(key) = &next_row.key
                {
                    parts.push(
                        StyledGraphemes::from(Self::render_yaml_string(key))
                            .apply_style(self.key_style),
                    );
                    parts.push(StyledGraphemes::from(": "));

                    if let Some(value) = self.render_node(&next_row.node) {
                        parts.push(value);
                    }

                    i += 1;
                }
            }

            let mut content: StyledGraphemes = parts.into_iter().collect();
            content = content.apply_attribute(if rendered_row == active_row {
                self.active_item_attribute
            } else {
                self.inactive_item_attribute
            });

            let mut line: StyledGraphemes = vec![indent, content].into_iter().collect();

            if let Some(width) = width {
                match self.overflow_mode {
                    OverflowMode::Truncate => {
                        line =
                            line.truncated_line_with_ellipsis(width, &StyledGraphemes::from("…"));
                        formatted.push(line);
                    }
                    OverflowMode::Wrap => {
                        formatted.extend(line.wrapped_lines(width));
                    }
                }
            } else {
                formatted.push(line);
            }

            i += 1;
            rendered_row += 1;
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod config {
        use super::*;

        mod render_terminal_rows {
            use super::*;
            use crate::structured::ContainerNode;

            #[test]
            fn renders_sequence_mapping_first_key_on_item_line() {
                let rows = vec![
                    Row {
                        depth: 1,
                        key: None,
                        is_sequence_item: true,
                        node: YamlNode::Container(ContainerNode::Open {
                            typ: ContainerType::Object,
                            collapsed: false,
                            close_index: 3,
                        }),
                    },
                    Row {
                        depth: 2,
                        key: Some("name".to_string()),
                        is_sequence_item: false,
                        node: YamlNode::String("alice".to_string()),
                    },
                    Row {
                        depth: 2,
                        key: Some("age".to_string()),
                        is_sequence_item: false,
                        node: YamlNode::Number(match serde_yaml::from_str("20").unwrap() {
                            serde_yaml::Value::Number(number) => number,
                            _ => unreachable!(),
                        }),
                    },
                ];

                let lines = Config {
                    indent: 2,
                    overflow_mode: OverflowMode::Truncate,
                    ..Default::default()
                }
                .render_terminal_rows(&rows, 80)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();

                assert_eq!(
                    lines,
                    vec!["- name: alice".to_string(), "  age: 20".to_string(),]
                );
            }

            #[test]
            fn does_not_indent_the_first_root_mapping_key() {
                let value = serde_yaml::from_str("name: alice\naddress:\n  city: Tokyo\n").unwrap();
                let document = crate::structured::yaml::Document::new([&value]);
                let rows = document.extract_rows_from_current(usize::MAX);

                let lines = Config {
                    indent: 2,
                    overflow_mode: OverflowMode::Truncate,
                    ..Default::default()
                }
                .render_terminal_rows(&rows, 80)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();

                assert_eq!(
                    lines,
                    vec![
                        "name: alice".to_string(),
                        "address: ".to_string(),
                        "  city: Tokyo".to_string(),
                    ]
                );
            }
        }
    }
}
