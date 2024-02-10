use std::{collections::HashSet, fmt, iter::FromIterator};

mod render;
pub use render::Renderer;
mod build;
pub use build::Builder;

use crate::core::menu::Menu;

#[derive(Clone)]
pub struct Checkbox {
    menu: Menu,
    picked: HashSet<usize>,
}

impl<T: fmt::Display> FromIterator<T> for Checkbox {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            menu: Menu::from_iter(iter),
            picked: HashSet::new(),
        }
    }
}

impl Checkbox {
    pub fn items(&self) -> &Vec<String> {
        self.menu.items()
    }

    pub fn position(&self) -> usize {
        self.menu.position()
    }

    pub fn picked_indexes(&self) -> &HashSet<usize> {
        &self.picked
    }

    pub fn get(&self) -> Vec<String> {
        self.picked
            .iter()
            .fold(Vec::<String>::new(), |mut ret, idx| {
                ret.push(self.menu.items().get(*idx).unwrap().to_owned());
                ret
            })
    }

    pub fn toggle(&mut self) {
        if self.picked.contains(&self.menu.position()) {
            self.picked.remove(&self.menu.position());
        } else {
            self.picked.insert(self.menu.position());
        }
    }

    pub fn backward(&mut self) -> bool {
        self.menu.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.menu.forward()
    }

    pub fn move_to_head(&mut self) {
        self.menu.move_to_head()
    }
}
