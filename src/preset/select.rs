use std::fmt::Display;

use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    listbox::{self, Listbox},
    render::{Renderable, State},
    style::Style,
    text, Prompt,
};

pub struct Select {
    title_renderer: text::Renderer,
    listbox_renderer: listbox::Renderer,
}

impl Select {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                screen_lines: Default::default(),
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
        self.listbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.active_item_style = style;
        self
    }

    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.inactive_item_style = style;
        self
    }

    pub fn screen_lines(mut self, screen_lines: usize) -> Self {
        self.listbox_renderer.screen_lines = Some(screen_lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<listbox::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .listbox
                    .get())
            },
        )
    }
}
