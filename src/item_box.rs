use std::{fmt, iter::FromIterator};

#[derive(Default)]
pub struct ItemBox {
    pub list: Vec<String>,
    pub position: usize,
}

impl<T: fmt::Display> FromIterator<T> for ItemBox {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            list: iter.into_iter().map(|e| format!("{}", e)).collect(),
            position: 0,
        }
    }
}

impl ItemBox {
    pub fn content(&self) -> Vec<String> {
        self.list.clone()
    }

    pub fn get(&self) -> String {
        self.list
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
        if !self.list.is_empty() && self.position < self.list.len() - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    pub fn to_head(&mut self) {
        self.position = 0
    }

    pub fn to_tail(&mut self) {
        self.position = self.list.len() - 1;
    }
}

#[cfg(test)]
mod test {
    mod backward {
        use super::super::*;

        #[test]
        fn test() {
            let mut ib = ItemBox::from_iter(["a", "b", "c"]);
            assert!(!ib.backward());
            ib.position = 1;
            assert!(ib.backward());
        }
    }

    mod forward {
        use super::super::*;

        #[test]
        fn test() {
            let mut ib = ItemBox::from_iter(["a", "b", "c"]);
            assert!(ib.forward());
            ib.position = ib.list.len() - 1;
            assert!(!ib.forward());
        }
    }
}
