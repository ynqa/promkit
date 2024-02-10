use std::{fmt::Display, iter::FromIterator};

use crate::{crossterm::style::ContentStyle, error::Result, render::State};

use super::{Checkbox, Renderer};

#[derive(Clone)]
pub struct Builder {
    checkbox: Checkbox,
    style: ContentStyle,
    mark: String,
    cursor: String,
    cursor_style: ContentStyle,
    lines: Option<usize>,
}

impl Builder {
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            checkbox: Checkbox::from_iter(items),
            style: Default::default(),
            mark: Default::default(),
            cursor: Default::default(),
            cursor_style: Default::default(),
            lines: Default::default(),
        }
    }

    pub fn mark<T: AsRef<str>>(mut self, mark: T) -> Self {
        self.mark = mark.as_ref().to_string();
        self
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.cursor = cursor.as_ref().to_string();
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

    pub fn build(self) -> Result<Renderer> {
        Ok(Renderer {
            checkbox: self.checkbox,
            mark: self.mark,
            style: self.style,
            cursor: self.cursor,
            cursor_style: self.cursor_style,
            lines: self.lines,
        })
    }

    pub fn build_state(self) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(self.build()?)))
    }
}
