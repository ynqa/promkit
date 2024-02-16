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
    text_editor::{self, Mode, Suggest, TextEditor},
    Prompt,
};

type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

pub struct Theme {
    /// Style for title (enabled if you set title).
    pub title_style: ContentStyle,

    /// Style for prompt string.
    pub ps_style: ContentStyle,
    /// Style for selected character.
    pub active_char_style: ContentStyle,
    /// Style for un-selected character.
    pub inactive_char_style: ContentStyle,

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
            ps_style: Style::new().fgc(Color::DarkGreen).build(),
            active_char_style: Style::new().bgc(Color::DarkCyan).build(),
            inactive_char_style: Style::new().build(),
            active_item_style: Style::new().fgc(Color::DarkCyan).build(),
            inactive_item_style: Style::new().build(),
        }
    }
}

pub struct QuerySelect {
    title: String,
    texteditor: TextEditor,
    listbox: Listbox,
    filter: Box<Filter>,
    theme: Theme,
    suggest: Suggest,
    ps: String,
    cursor: String,
    mode: Mode,
    text_editor_window_size: Option<usize>,
    listbox_window_size: Option<usize>,
}

impl QuerySelect {
    pub fn new<T, I, F>(items: I, filter: F) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
        F: Fn(&str, &Vec<String>) -> Vec<String> + 'static,
    {
        Self {
            title: Default::default(),
            texteditor: Default::default(),
            listbox: Listbox::from_iter(items),
            filter: Box::new(filter),
            theme: Default::default(),
            suggest: Default::default(),
            ps: String::from("❯❯ "),
            cursor: String::from("❯ "),
            mode: Default::default(),
            text_editor_window_size: Default::default(),
            listbox_window_size: Default::default(),
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

    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.suggest = suggest;
        self
    }

    pub fn prefix_string<T: AsRef<str>>(mut self, ps: T) -> Self {
        self.ps = ps.as_ref().to_string();
        self
    }

    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.cursor = cursor.as_ref().to_string();
        self
    }

    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn text_editor_window_size(mut self, window_size: usize) -> Self {
        self.text_editor_window_size = Some(window_size);
        self
    }

    pub fn listbox_window_size(mut self, window_size: usize) -> Self {
        self.listbox_window_size = Some(window_size);
        self
    }

    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            vec![
                State::<text::Renderer>::try_new(self.title, self.theme.title_style)?,
                State::<text_editor::Renderer>::try_new(
                    self.texteditor,
                    None,
                    self.suggest,
                    self.ps,
                    None,
                    self.theme.ps_style,
                    self.theme.active_char_style,
                    self.theme.inactive_char_style,
                    self.mode,
                    self.text_editor_window_size,
                )?,
                State::<listbox::Renderer>::try_new(
                    self.listbox,
                    self.cursor,
                    self.theme.active_item_style,
                    self.theme.inactive_item_style,
                    self.listbox_window_size,
                )?,
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
