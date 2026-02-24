use std::collections::HashSet;

use promkit_core::crossterm::style::ContentStyle;

use super::Mode;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, Default)]
pub struct Config {
    pub prefix: String,
    pub mask: Option<char>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub prefix_style: ContentStyle,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub active_char_style: ContentStyle,
    #[cfg_attr(
        feature = "serde",
        serde(with = "termcfg::crossterm_config::content_style_serde")
    )]
    pub inactive_char_style: ContentStyle,
    pub edit_mode: Mode,
    pub word_break_chars: HashSet<char>,
    pub lines: Option<usize>,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    mod serde_compatibility {
        use std::collections::HashSet;

        use promkit_core::crossterm::style::{Attribute, Color};

        use super::super::{Config, Mode};

        #[test]
        fn config_fields_are_fully_loaded_from_toml() {
            let input = r#"
prefix = ">> "
mask = "*"
prefix_style = "fg=green,attr=bold"
active_char_style = "bg=darkcyan,attr=underlined"
inactive_char_style = "fg=grey"
edit_mode = "Overwrite"
word_break_chars = [" ", ".", "/"]
lines = 3
"#;
            let formatter: Config = toml::from_str(input).unwrap();

            assert_eq!(formatter.prefix, ">> ");
            assert_eq!(formatter.mask, Some('*'));
            assert_eq!(formatter.prefix_style.foreground_color, Some(Color::Green));
            assert!(formatter.prefix_style.attributes.has(Attribute::Bold));
            assert_eq!(
                formatter.active_char_style.background_color,
                Some(Color::DarkCyan),
            );
            assert!(
                formatter
                    .active_char_style
                    .attributes
                    .has(Attribute::Underlined)
            );
            assert_eq!(
                formatter.inactive_char_style.foreground_color,
                Some(Color::Grey),
            );
            assert!(matches!(formatter.edit_mode, Mode::Overwrite));
            assert_eq!(formatter.word_break_chars, HashSet::from([' ', '.', '/']));
            assert_eq!(formatter.lines, Some(3));
        }
    }
}

pub type Formatter = Config;
