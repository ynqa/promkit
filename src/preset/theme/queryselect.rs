use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    style::Style,
};

pub struct Theme {
    pub title_style: ContentStyle,
    pub text_style: ContentStyle,
    pub item_style: ContentStyle,
    pub prefix: String,
    pub prefix_style: ContentStyle,
    pub listbox_cursor: String,
    pub listbox_cursor_style: ContentStyle,
    pub text_editor_cursor_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            text_style: Style::new().build(),
            item_style: Style::new().build(),
            prefix: String::from("❯❯ "),
            prefix_style: Style::new().fgc(Color::DarkGreen).build(),
            listbox_cursor: String::from("❯ "),
            listbox_cursor_style: Style::new().fgc(Color::DarkCyan).build(),
            text_editor_cursor_style: Style::new().bgc(Color::DarkCyan).build(),
        }
    }
}
