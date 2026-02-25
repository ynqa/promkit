use promkit_core::crossterm::style::ContentStyle;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone)]
pub struct Config {
    pub cursor: String,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::option_content_style_serde")
    )]
    pub active_item_style: Option<ContentStyle>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::option_content_style_serde")
    )]
    pub inactive_item_style: Option<ContentStyle>,
    pub lines: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cursor: String::from("❯ "),
            active_item_style: Some(ContentStyle::default()),
            inactive_item_style: Some(ContentStyle::default()),
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
cursor = "> "
active_item_style = "fg=cyan,attr=bold"
inactive_item_style = "fg=grey"
lines = 8
"#;

            let formatter: Config = toml::from_str(input).unwrap();
            assert_eq!(formatter.cursor, "> ");
            let active = formatter.active_item_style.unwrap();
            let inactive = formatter.inactive_item_style.unwrap();

            assert_eq!(active.foreground_color, Some(Color::Cyan));
            assert!(active.attributes.has(Attribute::Bold));
            assert_eq!(inactive.foreground_color, Some(Color::Grey));
            assert_eq!(formatter.lines, Some(8));
        }
    }
}
