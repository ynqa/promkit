use std::collections::VecDeque;

use crate::core::cursor::Cursor;

/// Manages the history of user inputs for a text editor.
/// This structure allows for the storage,
/// retrieval, and navigation through past inputs.
/// It supports adding new entries,
/// checking for the existence of specific entries,
/// and moving through the history in both forward and backward directions.
/// Additionally, it can limit the number of entries stored in the history
/// to a specified maximum size.
#[derive(Clone)]
pub struct History {
    /// Buffer storing the history of inputs as strings.
    pub cursor: Cursor<VecDeque<String>>,

    /// Optional limit on the number of entries in the history.
    /// If set, the history will not exceed this number of entries,
    /// and older entries will be removed to make room for new ones.
    limit_size: Option<usize>,
}

impl Default for History {
    /// Creates a new `History` instance with a single empty string in the buffer
    /// and initializes the position at 0.
    fn default() -> Self {
        Self {
            cursor: Cursor::new(VecDeque::from([String::new()])),
            limit_size: None,
        }
    }
}

impl History {
    pub fn new_with_limit_size(limit_size: usize) -> Self {
        Self {
            cursor: Cursor::new(VecDeque::from([String::new()])),
            limit_size: Some(limit_size),
        }
    }

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
            let init_state = self.cursor.contents_mut().pop_back().unwrap();
            self.cursor.contents_mut().push_back(item);
            if let Some(limit) = self.limit_size {
                if limit < self.cursor.contents_mut().len() {
                    self.cursor.contents_mut().pop_front();
                }
            }
            self.cursor.contents_mut().push_back(init_state);
        }
        self.move_to_tail();
    }

    /// Retrieves the current item from the history
    /// based on the current position.
    /// Returns an empty string if the position is out of bounds.
    pub fn get(&self) -> String {
        self.cursor
            .contents()
            .get(self.cursor.position())
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
        self.cursor.contents().iter().any(|i| i == item.as_ref())
    }

    /// Moves the current position backward in the history, if possible.
    /// Returns `true` if the position was successfully moved backward, `false` otherwise.
    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    /// Moves the current position forward in the history, if possible.
    /// Returns `true` if the position was successfully moved forward, `false` otherwise.
    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the current position to the tail (end) of the history buffer.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
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
            assert_eq!(
                VecDeque::from([String::from("item"), String::new()]),
                *h.cursor.contents()
            );
        }

        #[test]
        fn test_with_multiple_items() {
            let mut h = History::default();
            h.insert("item1");
            h.insert("item2");
            assert_eq!(
                VecDeque::from([String::from("item1"), String::from("item2"), String::new()]),
                *h.cursor.contents()
            );
        }

        #[test]
        fn test_with_limit_size() {
            let mut h = History::new_with_limit_size(2);
            h.insert("item1");
            h.insert("item2");
            h.insert("item3");
            assert_eq!(
                VecDeque::from([String::from("item2"), String::from("item3"), String::new()]),
                *h.cursor.contents()
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
