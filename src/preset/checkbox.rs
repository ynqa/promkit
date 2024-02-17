use std::fmt::Display;

use crate::{
    checkbox,
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    render::{Renderable, State},
    style::Style,
    text, Prompt,
};

pub struct Checkbox {
    title_renderer: text::Renderer,
    checkbox_renderer: checkbox::Renderer,
}

impl Checkbox {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            checkbox_renderer: checkbox::Renderer {
                checkbox: checkbox::Checkbox::from_iter(items),
                cursor: String::from("❯ "),
                mark: '■',
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                lines: Default::default(),
            },
        }
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.checkbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    pub fn mark(mut self, mark: char) -> Self {
        self.checkbox_renderer.mark = mark;
        self
    }

    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.active_item_style = style;
        self
    }

    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.inactive_item_style = style;
        self
    }

    pub fn checkbox_lines(mut self, lines: usize) -> Self {
        self.checkbox_renderer.lines = Some(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<checkbox::Renderer>::new(self.checkbox_renderer)),
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
