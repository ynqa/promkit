use promkit_core::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
};

use crate::structured::yaml::yamlz::{CollectionKind, Row, YamlNode};

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

    fn render_key(key: &str) -> String {
        Self::render_yaml_string(key)
    }

    fn render_collection_marker(&self, kind: &CollectionKind) -> StyledGraphemes {
        let style = match kind {
            CollectionKind::Mapping => self.map_style,
            CollectionKind::Sequence => self.sequence_style,
        };
        StyledGraphemes::from(kind.empty_str()).apply_style(style)
    }

    fn render_collapsed_marker(&self, kind: &CollectionKind) -> StyledGraphemes {
        let style = match kind {
            CollectionKind::Mapping => self.map_style,
            CollectionKind::Sequence => self.sequence_style,
        };
        StyledGraphemes::from(kind.collapsed_preview()).apply_style(style)
    }

    fn render_value(&self, row: &Row) -> Option<StyledGraphemes> {
        match &row.node {
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
            YamlNode::Empty { kind } => Some(self.render_collection_marker(kind)),
            YamlNode::Start {
                kind,
                collapsed: true,
                ..
            } => Some(self.render_collapsed_marker(kind)),
            YamlNode::Start {
                collapsed: false, ..
            } => {
                if row.key.is_none() && !row.is_sequence_item && row.depth == 0 {
                    Some(StyledGraphemes::from("---"))
                } else {
                    None
                }
            }
            YamlNode::End { .. } => None,
        }
    }

    /// Format YAML rows into terminal lines with styling and width constraints.
    pub fn format_for_terminal_display(&self, rows: &[Row], width: u16) -> Vec<StyledGraphemes> {
        let mut formatted = Vec::new();
        let width = width as usize;

        for (i, row) in rows.iter().enumerate() {
            let indent = StyledGraphemes::from(" ".repeat(self.indent * row.depth));
            let mut parts: Vec<StyledGraphemes> = Vec::new();

            if row.is_sequence_item {
                parts.push(StyledGraphemes::from("- "));
            }

            if let Some(key) = &row.key {
                parts
                    .push(StyledGraphemes::from(Self::render_key(key)).apply_style(self.key_style));
                parts.push(StyledGraphemes::from(": "));
            }

            if let Some(tag) = &row.tag {
                parts.push(StyledGraphemes::from(format!("{} ", tag)).apply_style(self.tag_style));
            }

            if let Some(value) = self.render_value(row) {
                parts.push(value);
            }

            let mut content: StyledGraphemes = parts.into_iter().collect();
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
