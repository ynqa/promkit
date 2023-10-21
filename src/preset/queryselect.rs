use std::fmt::Display;

use crate::{
    components::{
        Component, ItemPicker, ItemPickerBuilder, Mode, State, TextBuilder, TextEditor,
        TextEditorBuilder,
    },
    crossterm::event::Event,
    error::Result,
    item_box::ItemBox,
    suggest::Suggest,
    theme::queryselect::Theme,
    Prompt, PromptBuilder,
};

type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

pub struct QuerySelect {
    title: TextBuilder,
    text_editor: TextEditorBuilder,
    item_picker: ItemPickerBuilder,
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
            title: Default::default(),
            text_editor: Default::default(),
            item_picker: ItemPickerBuilder::new(items),
            filter: Box::new(filter),
        }
        .theme(Theme::default())
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.title = self.title.style(theme.title_style);
        self.text_editor = self
            .text_editor
            .prefix(theme.prefix)
            .prefix_style(theme.prefix_style)
            .style(theme.text_style)
            .cursor_style(theme.cursor_style);
        self.item_picker = self
            .item_picker
            .cursor(theme.cursor)
            .style(theme.item_style)
            .cursor_style(theme.cursor_style);
        self
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title = self.title.text(text);
        self
    }

    pub fn text_edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor = self.text_editor.edit_mode(mode);
        self
    }

    pub fn text_lines(mut self, lines: usize) -> Self {
        self.text_editor = self.text_editor.lines(lines);
        self
    }

    pub fn item_lines(mut self, lines: usize) -> Self {
        self.item_picker = self.item_picker.lines(lines);
        self
    }

    pub fn suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor = self.text_editor.suggest(suggest);
        self
    }

    pub fn prompt(self) -> Result<Prompt> {
        let filter = self.filter;

        PromptBuilder::new(vec![
            self.title.build_state()?,
            self.text_editor.build_state()?,
            self.item_picker.build_state()?,
        ])
        .evaluate(
            move |_: &Event, components: &Vec<Box<dyn Component + 'static>>| -> Result<bool> {
                let texteditor_state = components[1]
                    .as_any()
                    .downcast_ref::<State<TextEditor>>()
                    .unwrap();
                let itempicker_state = components[2]
                    .as_any()
                    .downcast_ref::<State<ItemPicker>>()
                    .unwrap();

                if texteditor_state.text_changed() {
                    let query = texteditor_state.after.borrow().output();

                    let list = filter(&query, &itempicker_state.init.itembox.list);
                    itempicker_state.after.borrow_mut().itembox = ItemBox { list, position: 0 };
                }
                Ok(true)
            },
        )
        .build()
    }
}
