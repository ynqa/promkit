use anyhow::Result;

use crate::{crossterm::style::ContentStyle, item_box::ItemBox, widgets::item_picker::ItemPicker};

pub struct ItemPickerBuilder {
    itembox: ItemBox,
    label: String,
    label_style: ContentStyle,
    style: ContentStyle,
    cursor_style: ContentStyle,
    lines: Option<usize>,
}

impl ItemPickerBuilder {
    pub fn new(itembox: ItemBox) -> Self {
        Self {
            itembox,
            label: String::from("❯ "),
            label_style: ContentStyle::new(),
            style: ContentStyle::new(),
            cursor_style: ContentStyle::new(),
            lines: None,
        }
    }

    pub fn label<T: AsRef<str>>(mut self, label: T) -> Self {
        self.label = label.as_ref().to_string();
        self
    }

    pub fn label_style(mut self, style: ContentStyle) -> Self {
        self.label_style = style;
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

    pub fn build(self) -> Result<Box<ItemPicker>> {
        Ok(Box::new(ItemPicker {
            itembox: self.itembox,
            label: self.label,
            label_style: self.label_style,
            style: self.style,
            cursor_style: self.cursor_style,
            lines: self.lines,
        }))
    }
}
