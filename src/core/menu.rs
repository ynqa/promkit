use std::{fmt, iter::FromIterator};

mod render;
pub use render::Renderer;
mod build;
pub use build::Builder;

use crate::core::cursor::Cursor;

#[derive(Clone)]
pub struct Menu(Cursor<Vec<String>>);

impl<T: fmt::Display> FromIterator<T> for Menu {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Cursor::new(
            iter.into_iter().map(|e| format!("{}", e)).collect(),
        ))
    }
}

impl Menu {
    pub fn items(&self) -> &Vec<String> {
        self.0.items()
    }

    pub fn position(&self) -> usize {
        self.0.position()
    }

    pub fn get(&self) -> String {
        self.0.get().unwrap_or(&String::new()).to_string()
    }

    pub fn backward(&mut self) -> bool {
        self.0.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.0.forward()
    }

    pub fn move_to_head(&mut self) {
        self.0.move_to_head()
    }

    pub fn move_to_tail(&mut self) {
        self.0.move_to_tail()
    }
}
