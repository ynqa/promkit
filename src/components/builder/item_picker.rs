use std::{fmt::Display, iter::FromIterator};

use crate::{crossterm::style::ContentStyle, error::Result, item_box::ItemBox};

use super::super::{item_picker::ItemPicker, State};

#[derive(Clone)]
pub struct ItemPickerBuilder {
    itembox: ItemBox,
    style: ContentStyle,
    cursor: String,
    cursor_style: ContentStyle,
    lines: Option<usize>,
}

impl ItemPickerBuilder {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            itembox: ItemBox::from_iter(items),
            style: Default::default(),
            cursor: Default::default(),
            cursor_style: Default::default(),
            lines: Default::default(),
        }
    }

    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.cursor = cursor.as_ref().to_string();
        self
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: ContentStyle) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = Some(lines);
        self
    }

    pub fn build(self) -> Result<ItemPicker> {
        Ok(ItemPicker {
            itembox: self.itembox,
            cursor: self.cursor,
            style: self.style,
            cursor_style: self.cursor_style,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<ItemPicker>>> {
        Ok(Box::new(State::<ItemPicker>::new(self.build()?)))
    }
}