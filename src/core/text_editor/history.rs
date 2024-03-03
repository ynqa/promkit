use std::collections::VecDeque;

/// Manages the history of user inputs for a text editor.
/// This structure allows for the storage,
/// retrieval, and navigation through past inputs.
/// It supports adding new entries,
/// checking for the existence of specific entries,
/// and moving through the history in both forward and backward directions.
#[derive(Clone)]
pub struct History {
    /// Buffer storing the history of inputs as strings.
    buf: VecDeque<String>,
    /// Current position in the history buffer.
    position: usize,
}

impl Default for History {
    /// Creates a new `History` instance with a single empty string in the buffer
    /// and initializes the position at 0.
    fn default() -> Self {
        Self {
            buf: VecDeque::from([String::new()]), // Start with an empty input as the current state.
            position: 0,
        }
    }
}

impl History {
    /// Inserts a new item into the history.
    ///
    /// If the item does not already exist in the buffer,
    /// it is inserted just before the last item.
    /// This method ensures there is always an empty string
    /// at the end of the buffer to represent
    /// a new input line. After insertion,
    /// the current position is moved to the end of the buffer.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to be inserted into the history.
    ///
    /// # Examples
    ///
    /// - Initial state: `items = [""]`
    /// - After inserting "abc": `items = ["abc", ""]`
    /// - After inserting "xyz": `items = ["abc", "xyz", ""]`
    pub fn insert<T: AsRef<str>>(&mut self, item: T) {
        let item = item.as_ref().to_string();
        if !self.exists(&item) {
            let init_state = self.buf.pop_back().unwrap();
            self.buf.push_back(item);
            self.buf.push_back(init_state);
        }
        self.move_to_tail();
    }

    /// Retrieves the current item from the history
    /// based on the current position.
    /// Returns an empty string if the position is out of bounds.
    pub fn get(&self) -> String {
        self.buf
            .get(self.position)
            .unwrap_or(&String::new())
            .to_string()
    }

    /// Checks whether a specific item exists in the history.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to check for existence in the history.
    ///
    /// # Returns
    ///
    /// Returns `true` if the item exists in the history, `false` otherwise.
    fn exists<T: AsRef<str>>(&self, item: T) -> bool {
        self.buf.iter().any(|i| i == item.as_ref())
    }

    /// Moves the current position backward in the history, if possible.
    /// Returns `true` if the position was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        if self.position > 0 {
            self.position -= 1;
            return true;
        }
        false
    }

    /// Moves the current position forward in the history, if possible.
    /// Returns `true` if the position was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        if !self.buf.is_empty() && self.position < self.buf.len() - 1 {
            self.position += 1;
            return true;
        }
        false
    }

    /// Moves the current position to the tail (end) of the history buffer.
    pub fn move_to_tail(&mut self) {
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
            h.insert("item");
            assert_eq!(VecDeque::from([String::from("item"), String::new()]), h.buf);
        }

        #[test]
        fn test_with_multiple_items() {
            let mut h = History::default();
            h.insert("item1");
            h.insert("item2");
            assert_eq!(
                VecDeque::from([String::from("item1"), String::from("item2"), String::new()]),
                h.buf
            );
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
