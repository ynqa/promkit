use std::{fmt, iter::FromIterator};

mod render;
pub use render::Renderer;
mod build;
pub use build::Builder;

#[derive(Clone, Default)]
pub struct Menu {
    items: Vec<String>,
    position: usize,
}

impl<T: fmt::Display> FromIterator<T> for Menu {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().map(|e| format!("{}", e)).collect(),
            position: 0,
        }
    }
}

impl Menu {
    pub fn items(&self) -> &Vec<String> {
        &self.items
    }

    pub fn get(&self) -> String {
        self.items
            .get(self.position)
            .unwrap_or(&String::new())
            .to_string()
    }

    pub fn backward(&mut self) -> bool {
        if 0 < self.position {
            self.position -= 1;
            return true;
        }
        false
    }

    pub fn forward(&mut self) -> bool {
        if !self.items.is_empty() && self.position < self.items.len() - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    pub fn move_to_head(&mut self) {
        self.position = 0
    }

    pub fn move_to_tail(&mut self) {
        self.position = self.items.len() - 1;
    }
}

#[cfg(test)]
mod test {
    mod backward {
        use super::super::*;

        #[test]
        fn test() {
            let mut b = Menu::from_iter(["a", "b", "c"]);
            assert!(!b.backward());
            b.position = 1;
            assert!(b.backward());
        }
    }

    mod forward {
        use super::super::*;

        #[test]
        fn test() {
            let mut b = Menu::from_iter(["a", "b", "c"]);
            assert!(b.forward());
            b.position = b.items.len() - 1;
            assert!(!b.forward());
        }
    }
}