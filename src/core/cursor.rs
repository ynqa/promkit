use crate::core::Len;

#[derive(Clone)]
pub struct Cursor<C> {
    contents: C,
    position: usize,
}

impl<C: Len> Cursor<C> {
    pub fn new(contents: C) -> Self {
        Self {
            contents,
            position: 0,
        }
    }

    pub fn contents(&self) -> &C {
        &self.contents
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn backward(&mut self) -> bool {
        if 0 < self.position {
            self.position -= 1;
            return true;
        }
        false
    }

    pub fn forward(&mut self) -> bool {
        let l = self.contents.len();
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
        let l = self.contents.len();
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
            b.position = b.contents.len() - 1;
            assert!(!b.forward());
        }
    }
}
