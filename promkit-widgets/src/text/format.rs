use promkit_core::crossterm::style::ContentStyle;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Default)]
pub struct Formatter {
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::option_content_style_serde")
    )]
    pub style: Option<ContentStyle>,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    mod serde_compatibility {
        use promkit_core::crossterm::style::{Attribute, Color};

        use super::super::Formatter;

        #[test]
        fn formatter_fields_are_fully_loaded_from_toml() {
            let input = r#"style = "fg=yellow,attr=bold""#;

            let formatter: Formatter = toml::from_str(input).unwrap();
            let style = formatter.style.unwrap();

            assert_eq!(style.foreground_color, Some(Color::Yellow));
            assert!(style.attributes.has(Attribute::Bold));
        }
    }
}
