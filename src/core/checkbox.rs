use std::{collections::HashSet, fmt, iter::FromIterator};

use crate::core::listbox::Listbox;

mod render;
pub use render::Renderer;

#[derive(Clone)]
pub struct Checkbox {
    listbox: Listbox,
    picked: HashSet<usize>,
}

impl<T: fmt::Display> FromIterator<T> for Checkbox {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            listbox: Listbox::from_iter(iter),
            picked: HashSet::new(),
        }
    }
}

impl Checkbox {
    pub fn items(&self) -> &Vec<String> {
        self.listbox.items()
    }

    pub fn position(&self) -> usize {
        self.listbox.position()
    }

    pub fn picked_indexes(&self) -> &HashSet<usize> {
        &self.picked
    }

    pub fn get(&self) -> Vec<String> {
        self.picked
            .iter()
            .fold(Vec::<String>::new(), |mut ret, idx| {
                ret.push(self.listbox.items().get(*idx).unwrap().to_owned());
                ret
            })
    }

    pub fn toggle(&mut self) {
        if self.picked.contains(&self.listbox.position()) {
            self.picked.remove(&self.listbox.position());
        } else {
            self.picked.insert(self.listbox.position());
        }
    }

    pub fn backward(&mut self) -> bool {
        self.listbox.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.listbox.forward()
    }

    pub fn move_to_head(&mut self) {
        self.listbox.move_to_head()
    }
}
