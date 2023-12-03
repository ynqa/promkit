use std::fmt::Display;

use crate::{
    crossterm::event::Event,
    error::Result,
    preset::theme::queryselect::Theme,
    render::{Renderable, State},
    select_box::SelectBox,
    select_box::{Builder as SelectBoxRendererBuilder, Renderer as SelectBoxRenderer},
    text::Builder as TextRendererBuilder,
    text_editor::{
        Builder as TextEditorRendererBuilder, Mode, Renderer as TextEditorRenderer, Suggest,
    },
    Prompt,
};

type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

pub struct QuerySelect {
    title_builder: TextRendererBuilder,
    text_editor_builder: TextEditorRendererBuilder,
    select_builder: SelectBoxRendererBuilder,
    filter: Box<Filter>,
}

impl QuerySelect {
    pub fn new<T, I, F>(items: I, filter: F) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
        F: Fn(&str, &Vec<String>) -> Vec<String> + 'static,
    {
        Self {
            title_builder: Default::default(),
            text_editor_builder: Default::default(),
            select_builder: SelectBoxRendererBuilder::new(items),
            filter: Box::new(filter),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title_builder = self.title_builder.style(theme.title_style);
        self.text_editor_builder = self
            .text_editor_builder
            .prefix(theme.prefix)
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style);
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

    pub fn text_edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_builder = self.text_editor_builder.edit_mode(mode);
        self
    }

    pub fn text_lines(mut self, lines: usize) -> Self {
        self.text_editor_builder = self.text_editor_builder.lines(lines);
        self
    }

    pub fn item_lines(mut self, lines: usize) -> Self {
        self.select_builder = self.select_builder.lines(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_builder = self.text_editor_builder.suggest(suggest);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            vec![
                self.title_builder.build_state()?,
                self.text_editor_builder.build_state()?,
                self.select_builder.build_state()?,
            ],
            move |_: &Event, renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<bool> {
                let text_editor_state = renderables[1]
                    .as_any()
                    .downcast_ref::<State<TextEditorRenderer>>()
                    .unwrap();
                let select_state = renderables[2]
                    .as_any()
                    .downcast_ref::<State<SelectBoxRenderer>>()
                    .unwrap();

                if text_editor_state.text_changed() {
                    let query = text_editor_state
                        .after
                        .borrow()
                        .texteditor
                        .content_without_cursor();

                    let list = filter(&query, &select_state.init.selectbox.list);
                    select_state.after.borrow_mut().selectbox = SelectBox { list, position: 0 };
                }
                Ok(true)
            },
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[2]
                    .as_any()
                    .downcast_ref::<State<SelectBoxRenderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .selectbox
                    .get())
            },
        )
    }
}
