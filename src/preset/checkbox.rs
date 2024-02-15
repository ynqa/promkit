use std::fmt::Display;

use crate::{
    checkbox,
    error::Result,
    preset::theme::checkbox::Theme,
    render::{Renderable, State},
    text, Prompt,
};

pub struct Checkbox {
    title_builder: text::Builder,
    checkbox_builder: checkbox::Builder,
}

impl Checkbox {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_builder: Default::default(),
            checkbox_builder: checkbox::Builder::new(items),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.checkbox_builder = self
            .checkbox_builder
            .mark(theme.mark)
            .cursor(theme.cursor)
            .style(theme.item_style)
            .cursor_style(theme.cursor_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_builder = self.title_builder.text(text);
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.checkbox_builder = self.checkbox_builder.lines(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            vec![
                self.title_builder.build_state()?,
                self.checkbox_builder.build_state()?,
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
