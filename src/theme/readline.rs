use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    style::Style,
};

pub struct Theme {
    pub title_style: ContentStyle,
    pub label: String,
    pub label_style: ContentStyle,
    pub text_style: ContentStyle,
    pub cursor_style: ContentStyle,
    pub error_message_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            label: String::from("❯❯ "),
            label_style: Style::new().fgc(Color::DarkGreen).build(),
            text_style: Style::new().build(),
            cursor_style: Style::new().bgc(Color::DarkCyan).build(),
            error_message_style: Style::new()
                .fgc(Color::DarkRed)
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
        }
    }
}
