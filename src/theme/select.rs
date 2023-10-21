use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    style::Style,
};

pub struct Theme {
    pub title_style: ContentStyle,
    pub label: String,
    pub item_style: ContentStyle,
    pub cursor_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            label: String::from("❯ "),
            item_style: Style::new().build(),
            cursor_style: Style::new().fgc(Color::DarkCyan).build(),
        }
    }
}
