use std::fmt::Display;

use crate::{
    error::Result,
    listbox::{Builder as ListboxRendererBuilder, Renderer as ListboxRenderer},
    preset::theme::select::Theme,
    render::{Renderable, State},
    text::Builder as TextRendererBuilder,
    Prompt,
};

pub struct Select {
    title_builder: TextRendererBuilder,
    listbox_builder: ListboxRendererBuilder,
}

impl Select {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_builder: Default::default(),
            listbox_builder: ListboxRendererBuilder::new(items),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.listbox_builder = self
            .listbox_builder
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
        self.listbox_builder = self.listbox_builder.lines(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                self.title_builder.build_state()?,
                self.listbox_builder.build_state()?,
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<ListboxRenderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .listbox
                    .get())
            },
        )
    }
}
