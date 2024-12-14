use crate::{
    crossterm::style::{Attribute, ContentStyle},
    grapheme::StyledGraphemes,
    pane::Pane,
    PaneFactory,
};

use super::JsonStream;

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
    /// Formats a Vec<Row> into Vec<StyledGraphemes> with appropriate styling and width limits
    fn format_rows(&self, rows: Vec<super::jsonz::Row>, width: u16) -> Vec<StyledGraphemes> {
        let mut formatted = Vec::new();
        let width = width as usize;

        for (i, row) in rows.iter().enumerate() {
            let mut parts = Vec::new();

            parts.push(StyledGraphemes::from(" ".repeat(self.indent * row.depth)));

            if let Some(key) = &row.k {
                parts.push(
                    StyledGraphemes::from(format!("\"{}\"", key)).apply_style(self.key_style),
                );
                parts.push(StyledGraphemes::from(": "));
            }

            match &row.v {
                super::jsonz::Value::Null => {
                    parts.push(StyledGraphemes::from("null").apply_style(self.null_value_style));
                }
                super::jsonz::Value::Boolean(b) => {
                    parts.push(
                        StyledGraphemes::from(b.to_string()).apply_style(self.boolean_value_style),
                    );
                }
                super::jsonz::Value::Number(n) => {
                    parts.push(
                        StyledGraphemes::from(n.to_string()).apply_style(self.number_value_style),
                    );
                }
                super::jsonz::Value::String(s) => {
                    let escaped = s.replace('\n', "\\n");
                    parts.push(
                        StyledGraphemes::from(format!("\"{}\"", escaped))
                            .apply_style(self.string_value_style),
                    );
                }
                super::jsonz::Value::Empty { typ } => {
                    let bracket_style = match typ {
                        super::jsonz::ContainerType::Object => self.curly_brackets_style,
                        super::jsonz::ContainerType::Array => self.square_brackets_style,
                    };
                    parts.push(
                        StyledGraphemes::from(format!("{}{}", typ.open_str(), typ.close_str()))
                            .apply_style(bracket_style),
                    );
                }
                super::jsonz::Value::Open { typ, collapsed, .. } => {
                    let bracket_style = match typ {
                        super::jsonz::ContainerType::Object => self.curly_brackets_style,
                        super::jsonz::ContainerType::Array => self.square_brackets_style,
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
                super::jsonz::Value::Close { typ, .. } => {
                    let bracket_style = match typ {
                        super::jsonz::ContainerType::Object => self.curly_brackets_style,
                        super::jsonz::ContainerType::Array => self.square_brackets_style,
                    };
                    parts.push(StyledGraphemes::from(typ.close_str()).apply_style(bracket_style));
                }
            }

            if i + 1 < rows.len() {
                if let super::jsonz::Value::Close { .. } = rows[i + 1].v {
                } else if let super::jsonz::Value::Open { .. } = rows[i].v {
                } else {
                    parts.push(StyledGraphemes::from(","));
                }
            }

            let mut line: StyledGraphemes = parts.into_iter().collect();

            if line.widths() > width {
                let mut truncated = StyledGraphemes::default();
                let mut current_width = 0;
                for g in line.iter() {
                    if current_width + g.width() + 3 > width {
                        break;
                    }
                    truncated.push_back(g.clone());
                    current_width += g.width();
                }
                let ellipsis: StyledGraphemes = StyledGraphemes::from("...");
                line = vec![truncated, ellipsis].into_iter().collect();
            }

            // Note that `extract_rows_from_current`
            // returns rows starting from the current position,
            // so the first row should always be highlighted as active
            line = line.apply_attribute(if i == 0 {
                self.active_item_attribute
            } else {
                self.inactive_item_attribute
            });

            formatted.push(line);
        }

        formatted
    }
}

impl PaneFactory for State {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        let height = match self.lines {
            Some(lines) => lines.min(height as usize),
            None => height as usize,
        };

        let rows = self.stream.extract_rows_from_current(height);
        let formatted_rows = self.format_rows(rows, width);

        Pane::new(formatted_rows, 0)
    }
}
