#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub mod yaml;

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
