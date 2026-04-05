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

/// Structured row node for container representation.
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerNode {
    Empty {
        typ: ContainerType,
    },
    Open {
        typ: ContainerType,
        collapsed: bool,
        close_index: usize,
    },
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
