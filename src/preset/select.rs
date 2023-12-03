use std::fmt::Display;

use crate::{
    error::Result,
    theme::select::Theme,
    view::{SelectViewer, SelectViewerBuilder, State, TextViewerBuilder, Viewable},
    Prompt,
};

pub struct Select {
    title_builder: TextViewerBuilder,
    select_builder: SelectViewerBuilder,
}

impl Select {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_builder: Default::default(),
            select_builder: SelectViewerBuilder::new(items),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.select_builder = self
            .select_builder
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
        self.select_builder = self.select_builder.lines(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            vec![
                self.title_builder.build_state()?,
                self.select_builder.build_state()?,
            ],
            |_, _| Ok(true),
            |viewables: &Vec<Box<dyn Viewable + 'static>>| -> Result<String> {
                Ok(viewables[1]
                    .as_any()
                    .downcast_ref::<State<SelectViewer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .selectbox
                    .get())
            },
        )
    }
}
