use std::fmt::Display;

use crate::{
    checkbox,
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    render::{Renderable, State},
    style::Style,
    text, Prompt,
};

pub struct Theme {
    /// Style for title (enabled if you set title).
    pub title_style: ContentStyle,

    /// Style for selected item.
    pub active_item_style: ContentStyle,
    /// Style for un-selected item.
    pub inactive_item_style: ContentStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .attrs(Attributes::from(Attribute::Bold))
                .build(),
            active_item_style: Style::new().fgc(Color::DarkCyan).build(),
            inactive_item_style: Style::new().build(),
        }
    }
}

pub struct Checkbox {
    title: String,
    checkbox: checkbox::Checkbox,
    theme: Theme,
    cursor: String,
    mark: char,
    window_size: Option<usize>,
}

impl Checkbox {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title: Default::default(),
            checkbox: checkbox::Checkbox::from_iter(items),
            theme: Default::default(),
            cursor: String::from("❯ "),
            mark: '■',
            window_size: Default::default(),
        }
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title = text.as_ref().to_string();
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.cursor = cursor.as_ref().to_string();
        self
    }

    pub fn mark(mut self, mark: char) -> Self {
        self.mark = mark;
        self
    }

    pub fn window_size(mut self, window_size: usize) -> Self {
        self.window_size = Some(window_size);
        self
    }

    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            vec![
                State::<text::Renderer>::try_new(self.title, self.theme.title_style)?,
                State::<checkbox::Renderer>::try_new(
                    self.checkbox,
                    self.cursor,
                    self.mark,
                    self.theme.active_item_style,
                    self.theme.inactive_item_style,
                    self.window_size,
                )?,
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<Vec<String>> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<checkbox::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .checkbox
                    .get())
            },
        )
    }
}
