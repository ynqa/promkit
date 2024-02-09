use std::ops::Deref;

#[derive(Clone)]
pub struct Cursor<I> {
    items: I,
    position: usize,
}

impl<T: Default, I: Deref<Target = [T]>> Cursor<I> {
    pub fn new(items: I) -> Self {
        Self { items, position: 0 }
    }

    pub fn items(&self) -> &I {
        &self.items
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn get(&self) -> Option<&T> {
        self.items.get(self.position)
    }

    pub fn backward(&mut self) -> bool {
        if 0 < self.position {
            self.position -= 1;
            return true;
        }
        false
    }

    pub fn forward(&mut self) -> bool {
        let l = self.items.len();
        if l != 0 && self.position < l - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    pub fn move_to_head(&mut self) {
        self.position = 0
    }

    pub fn move_to_tail(&mut self) {
        let l = self.items.len();
        if l == 0 {
            self.position = 0
        } else {
            self.position = l - 1;
        }
    }
}

#[cfg(test)]
mod test {

    mod backward {
        use super::super::*;

        #[test]
        fn test() {
            let mut b = Cursor::new(vec!["a", "b", "c"]);
            assert!(!b.backward());
            b.position = 1;
            assert!(b.backward());
        }
    }

    mod forward {
        use super::super::*;

        #[test]
        fn test() {
            let mut b = Cursor::new(vec!["a", "b", "c"]);
            assert!(b.forward());
            b.position = b.items.len() - 1;
            assert!(!b.forward());
        }
    }
}
