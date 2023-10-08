/// Store the histroy of the past user inputs.
pub struct History {
    buf: Vec<String>,
    position: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            buf: vec![String::new()],
            position: 0,
        }
    }
}

impl History {
    /// Insert an item.
    ///
    /// # NOTE
    ///
    /// Insert the items to the history with the following steps:
    ///
    /// 1. items = [""]
    /// 1. Input: "abc"
    /// 1. items = ["abc", ""]
    /// 1. Input "xyz"
    /// 1. items = ["abc", "xyz", ""]
    pub fn insert<T: AsRef<str>>(&mut self, item: T) {
        let item = item.as_ref().to_string();
        if self.buf.is_empty() {
            self.buf.push(item)
        } else {
            if !self.exists(&item) {
                let tail_idx = self.buf.len() - 1;
                self.buf.insert(tail_idx, item);
            }
            self.to_tail();
        }
    }

    pub fn get(&self) -> String {
        self.buf.get(self.position).unwrap().to_string()
    }

    /// Check whether the item exists or not.
    fn exists<T: AsRef<str>>(&self, item: T) -> bool {
        self.buf.iter().any(|i| i == item.as_ref())
    }

    pub fn prev(&mut self) -> bool {
        if 0 < self.position {
            self.position -= 1;
            return true;
        }
        false
    }

    pub fn next(&mut self) -> bool {
        if !self.buf.is_empty() && self.position < self.buf.len() - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    pub fn to_tail(&mut self) {
        self.position = self.buf.len() - 1;
    }
}

#[cfg(test)]
mod test {
    mod insert {
        use super::super::*;

        #[test]
        fn test() {
            let mut h = History::default();
            h.insert("line");
            assert_eq!(h.position, 1);
            assert_eq!(h.get(), "");
        }
    }

    mod exists {
        use super::super::*;

        #[test]
        fn test() {
            let mut h = History::default();
            h.insert("existed");
            assert!(h.exists("existed"));
            assert!(!h.exists("not_found"));
        }
    }
}
