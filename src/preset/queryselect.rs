use std::{fmt::Display, iter::FromIterator};

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    listbox::{self, Listbox},
    render::{Renderable, State},
    style::Style,
    text,
    text_editor::{self, Mode, Suggest},
    Prompt,
};

type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

pub struct QuerySelect {
    title_renderer: text::Renderer,
    text_editor_renderer: text_editor::Renderer,
    listbox_renderer: listbox::Renderer,
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
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: None,
                suggest: Default::default(),
                ps: String::from("❯❯ "),
                mask: None,
                ps_style: Style::new().fgc(Color::DarkGreen).build(),
                active_char_style: Style::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: Style::new().build(),
                edit_mode: Default::default(),
                lines: Default::default(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                lines: Default::default(),
            },
            filter: Box::new(filter),
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

    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_renderer.suggest = suggest;
        self
    }

    pub fn prefix_string<T: AsRef<str>>(mut self, ps: T) -> Self {
        self.text_editor_renderer.ps = ps.as_ref().to_string();
        self
    }

    pub fn prefix_string_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.ps_style = style;
        self
    }

    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.active_char_style = style;
        self
    }

    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.inactive_char_style = style;
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_renderer.edit_mode = mode;
        self
    }

    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_renderer.lines = Some(lines);
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

    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_renderer.lines = Some(lines);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(State::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            move |_: &Event, renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<bool> {
                let text_editor_state = renderables[1]
                    .as_any()
                    .downcast_ref::<State<text_editor::Renderer>>()
                    .unwrap();
                let select_state = renderables[2]
                    .as_any()
                    .downcast_ref::<State<listbox::Renderer>>()
                    .unwrap();

                if text_editor_state.text_changed() {
                    let query = text_editor_state
                        .after
                        .borrow()
                        .texteditor
                        .text_without_cursor();

                    let list = filter(&query, select_state.init.listbox.items());
                    select_state.after.borrow_mut().listbox = Listbox::from_iter(list);
                }
                Ok(true)
            },
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<String> {
                Ok(renderables[2]
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
