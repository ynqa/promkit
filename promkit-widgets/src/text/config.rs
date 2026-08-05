use promkit_core::crossterm::style::ContentStyle;

/// Defines how text is rendered when a line exceeds the available width.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowMode {
    /// Truncates lines and appends an ellipsis character (…).
    Truncate,
    /// Wraps lines onto subsequent visual rows.
    #[default]
    Wrap,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Default)]
pub struct Config {
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::option_content_style_serde")
    )]
    pub style: Option<ContentStyle>,
    pub lines: Option<usize>,
    pub overflow_mode: OverflowMode,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    mod deserialize {
        use promkit_core::crossterm::style::{Attribute, Color};

        use super::super::{Config, OverflowMode};

        #[test]
        fn loads_all_fields_from_toml() {
            let input = r#"
style = "fg=yellow,attr=bold"
lines = 2
overflow_mode = "Truncate"
"#;

            let formatter: Config = toml::from_str(input).unwrap();
            let style = formatter.style.unwrap();

            assert_eq!(style.foreground_color, Some(Color::Yellow));
            assert!(style.attributes.has(Attribute::Bold));
            assert_eq!(formatter.lines, Some(2));
            assert_eq!(formatter.overflow_mode, OverflowMode::Truncate);
        }

        #[test]
        fn uses_wrap_by_default() {
            let formatter: Config = toml::from_str("").unwrap();

            assert_eq!(formatter.overflow_mode, OverflowMode::Wrap);
        }
    }
}
