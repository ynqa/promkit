pub mod len;
use len::Len;

/// A generic cursor structure for navigating and manipulating collections.
/// It maintains a position within the collection
/// and provides methods to move forward, backward,
/// to the head, and to the tail of the collection.
/// It requires the collection to implement the `Len` trait.
/// The `cyclic` parameter allows the cursor to cycle through the collection.
#[derive(Clone)]
pub struct Cursor<C> {
    contents: C,
    position: usize,
    cyclic: bool,
}

impl<C: Len> Cursor<C> {
    /// Constructs a new `Cursor` with the given contents, an initial position, and a cyclic flag.
    /// If the given position is greater than the length of the contents,
    /// it sets the position to the last item of the contents.
    /// If `cyclic` is true, the cursor can cycle through the collection.
    pub fn new(contents: C, position: usize, cyclic: bool) -> Self {
        let adjusted_position = if position >= contents.len() {
            contents.len().saturating_sub(1)
        } else {
            position
        };

        Self {
            contents,
            position: adjusted_position,
            cyclic,
        }
    }

    /// Returns a reference to the contents.
    pub fn contents(&self) -> &C {
        &self.contents
    }

    /// Returns a mutable reference to the contents.
    pub fn contents_mut(&mut self) -> &mut C {
        &mut self.contents
    }

    /// Returns the current position of the cursor.
    pub fn position(&self) -> usize {
        self.position
    }

    pub fn shift(&mut self, backward: usize, forward: usize) -> bool {
        let len = self.contents.len();
        if self.cyclic {
            let total_move = forward as isize - backward as isize;
            let new_position =
                (self.position as isize + total_move).rem_euclid(len as isize) as usize;
            self.position = new_position;
            true
        } else if backward > self.position {
            false
        } else {
            let new_position = self.position - backward;
            if new_position + forward < len {
                self.position = new_position + forward;
                true
            } else {
                false
            }
        }
    }

    /// Moves the cursor one position backward, if possible. Returns `true` if successful.
    /// If `cyclic` is true and the cursor is at the head, it moves to the tail.
    pub fn backward(&mut self) -> bool {
        self.shift(1, 0)
    }

    /// Moves the cursor one position forward, if possible. Returns `true` if successful.
    /// If `cyclic` is true and the cursor is at the tail, it moves to the head.
    pub fn forward(&mut self) -> bool {
        self.shift(0, 1)
    }

    /// Moves the cursor to the head (start) of the contents.
    pub fn move_to_head(&mut self) {
        self.move_to(0);
    }

    /// Checks if the cursor is at the head (start) of the contents.
    pub fn is_head(&self) -> bool {
        self.position == 0
    }

    /// Moves the cursor to the tail (end) of the contents.
    pub fn move_to_tail(&mut self) {
        self.move_to(self.contents.len().saturating_sub(1));
    }

    /// Checks if the cursor is at the tail (end) of the contents.
    pub fn is_tail(&self) -> bool {
        self.position == self.contents.len().saturating_sub(1)
    }

    pub fn move_to(&mut self, position: usize) -> bool {
        if position < self.contents.len() {
            self.position = position;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod shift {
        use super::*;

        #[test]
        fn test_cyclic_forward() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 0, true);
            assert!(cursor.shift(0, 2)); // 0 -> 2
            assert_eq!(cursor.position(), 2);
        }

        #[test]
        fn test_cyclic_backward() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 2, true);
            assert!(cursor.shift(2, 0)); // 2 -> 0
            assert_eq!(cursor.position(), 0);
        }

        #[test]
        fn test_cyclic_wrap_around() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 2, true);
            assert!(cursor.shift(0, 1)); // 2 -> 0 (wrap around)
            assert_eq!(cursor.position(), 0);
        }

        #[test]
        fn test_non_cyclic_forward_fail() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 2, false);
            assert!(!cursor.shift(0, 1)); // 2 -> fail, no wrap around
        }

        #[test]
        fn test_non_cyclic_backward_fail() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 0, false);
            assert!(!cursor.shift(1, 0)); // 0 -> fail, can't move backward
        }

        #[test]
        fn test_non_cyclic_forward_success() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 1, false);
            assert!(cursor.shift(0, 1)); // 1 -> 2
            assert_eq!(cursor.position(), 2);
        }

        #[test]
        fn test_non_cyclic_backward_success() {
            let mut cursor = Cursor::new(vec!["a", "b", "c"], 2, false);
            assert!(cursor.shift(1, 0)); // 2 -> 1
            assert_eq!(cursor.position(), 1);
        }
    }

    mod backward {
        use super::*;

        #[test]
        fn test() {
            let mut b = Cursor::new(vec!["a", "b", "c"], 0, false);
            assert!(!b.backward());
            b.position = 1;
            assert!(b.backward());
        }
    }

    mod forward {
        use super::*;

        #[test]
        fn test() {
            let mut b = Cursor::new(vec!["a", "b", "c"], 0, false);
            assert!(b.forward());
            b.position = b.contents.len() - 1;
            assert!(!b.forward());
        }
    }
}
