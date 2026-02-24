use promkit_core::crossterm::style::{Color, ContentStyle};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone)]
pub struct Config {
    pub folded_symbol: String,
    pub unfolded_symbol: String,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub active_item_style: ContentStyle,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub inactive_item_style: ContentStyle,
    pub indent: usize,
    pub lines: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            folded_symbol: String::from("▶︎ "),
            unfolded_symbol: String::from("▼ "),
            active_item_style: ContentStyle {
                foreground_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            inactive_item_style: ContentStyle::default(),
            indent: 2,
            lines: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    mod serde_compatibility {
        use promkit_core::crossterm::style::{Attribute, Color};

        use super::super::Config;

        #[test]
        fn config_fields_are_fully_loaded_from_toml() {
            let input = r#"
folded_symbol = "> "
unfolded_symbol = "v "
active_item_style = "fg=cyan,attr=bold"
inactive_item_style = "fg=grey"
indent = 4
lines = 9
"#;
            let formatter: Config = toml::from_str(input).unwrap();

            assert_eq!(formatter.folded_symbol, "> ");
            assert_eq!(formatter.unfolded_symbol, "v ");
            assert_eq!(
                formatter.active_item_style.foreground_color,
                Some(Color::Cyan)
            );
            assert!(formatter.active_item_style.attributes.has(Attribute::Bold));
            assert_eq!(
                formatter.inactive_item_style.foreground_color,
                Some(Color::Grey),
            );
            assert_eq!(formatter.indent, 4);
            assert_eq!(formatter.lines, Some(9));
        }
    }
}
