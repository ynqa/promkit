use promkit_core::crossterm::style::{Attribute, ContentStyle};

/// Rendering configuration for a table.
#[derive(Clone)]
pub struct Config {
    /// Style applied to header cells.
    pub header_style: ContentStyle,
    /// Style applied to body cells.
    pub cell_style: ContentStyle,
    /// Style applied to separators.
    pub separator_style: ContentStyle,
    /// Attribute applied to the selected body row.
    pub active_item_attribute: Attribute,
    /// Text inserted between visible columns.
    pub separator: String,
    /// Maximum display width of one column.
    ///
    /// `None` keeps the widest cell width. The final visible column can still
    /// be clipped to the terminal viewport.
    pub max_column_width: Option<usize>,
    /// Maximum number of rendered rows, including the header.
    pub lines: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            header_style: ContentStyle::default(),
            cell_style: ContentStyle::default(),
            separator_style: ContentStyle::default(),
            active_item_attribute: Attribute::Reverse,
            separator: " │ ".to_owned(),
            max_column_width: Some(32),
            lines: None,
        }
    }
}
