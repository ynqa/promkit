#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub mod yaml;

#[cfg(feature = "tree")]
#[cfg_attr(docsrs, doc(cfg(feature = "tree")))]
pub mod tree;

/// Container type of structured widget.
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerType {
    Object,
    Array,
}

impl ContainerType {
    /// Oprening string of the container.
    pub fn open_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{",
            ContainerType::Array => "[",
        }
    }

    /// Closing string of the container.
    pub fn close_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "}",
            ContainerType::Array => "]",
        }
    }

    /// Empty string of the container.
    pub fn empty_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{}",
            ContainerType::Array => "[]",
        }
    }

    /// Collapsed preview string of the container.
    pub fn collapsed_preview(&self) -> &'static str {
        match self {
            ContainerType::Object => "{…}",
            ContainerType::Array => "[…]",
        }
    }
}

/// Container node of structured widget, which can be an empty container,
/// an opening container, or a closing container.
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerNode {
    /// An empty container (e.g., `{}` or `[]`).
    Empty { typ: ContainerType },
    /// An opening container (e.g., `{` or `[`), with information
    /// about whether it's collapsed and its corresponding closing index.
    Open {
        typ: ContainerType,
        collapsed: bool,
        close_index: usize,
    },
    /// A closing container (e.g., `}` or `]`), with information
    /// about whether it's collapsed and its corresponding opening index.
    Close {
        typ: ContainerType,
        collapsed: bool,
        open_index: usize,
    },
}

pub trait PrettyRender {
    /// Render the row as a pretty-printed string with the specified indentation level.
    fn render_pretty(&self, indent: usize) -> String;
}

/// Common row navigation and visibility operations for structured views.
pub trait RowOperation {
    /// Row element type handled by the operation target.
    type Row;

    /// Move the cursor one row upward from `current`.
    fn up(&self, current: usize) -> usize;

    /// Move the cursor to the first visible row.
    fn head(&self) -> usize;

    /// Move the cursor one row downward from `current`.
    fn down(&self, current: usize) -> usize;

    /// Move the cursor to the last visible row.
    fn tail(&self) -> usize;

    /// Toggle collapse/expand state at `current` and return the next cursor position.
    fn toggle(&mut self, current: usize) -> usize;

    /// Set collapsed visibility for all container rows.
    fn set_rows_visibility(&mut self, collapsed: bool);

    /// Extract up to `n` visible rows starting from `current`.
    fn extract(&self, current: usize, n: usize) -> Vec<Self::Row>;
}
