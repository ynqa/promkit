mod len;
use len::Len;

/// A generic cursor structure for navigating and manipulating collections.
/// It maintains a position within the collection
/// and provides methods to move forward, backward,
/// to the head, and to the tail of the collection.
/// It requires the collection to implement the `Len` trait.
#[derive(Clone)]
pub struct Cursor<C> {
    contents: C,
    position: usize,
}

impl<C: Len> Cursor<C> {
    /// Constructs a new `Cursor` with the given contents, initially positioned at the start.
    pub fn new(contents: C) -> Self {
        Self {
            contents,
            position: 0,
        }
    }

    /// Constructs a new `Cursor` with the given contents and an initial position.
    /// If the given position is greater than the length of the contents,
    /// it sets the position to the last item of the contents.
    pub fn new_with_position(contents: C, position: usize) -> Self {
        let adjusted_position = if position >= contents.len() {
            contents.len().saturating_sub(1)
        } else {
            position
        };

        Self {
            contents,
            position: adjusted_position,
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

    /// Replaces the current contents with the new contents provided.
    /// If the current position is beyond the end of the new contents,
    /// the cursor's position is adjusted to the last item of the new contents.
    pub fn replace(&mut self, new: C) {
        let new_tail = new.len().saturating_sub(1);
        if self.position() > new_tail {
            self.position = new_tail;
        }
        self.contents = new;
    }

    /// Moves the cursor one position backward, if possible. Returns `true` if successful.
    pub fn backward(&mut self) -> bool {
        if self.position > 0 {
            self.position = self.position.saturating_sub(1);
            return true;
        }
        false
    }

    /// Moves the cursor one position forward, if possible. Returns `true` if successful.
    pub fn forward(&mut self) -> bool {
        let l = self.contents.len();
        if l != 0 && self.position < l.saturating_sub(1) {
            self.position += 1;
            return true;
        }
        false
    }

    /// Moves the cursor to the head (start) of the contents.
    pub fn move_to_head(&mut self) {
        self.position = 0
    }

    /// Checks if the cursor is at the head (start) of the contents.
    pub fn is_head(&self) -> bool {
        self.position == 0
    }

    /// Moves the cursor to the tail (end) of the contents.
    pub fn move_to_tail(&mut self) {
        let l = self.contents.len();
        if l == 0 {
            self.position = 0
        } else {
            self.position = l.saturating_sub(1);
        }
    }

    /// Checks if the cursor is at the tail (end) of the contents.
    pub fn is_tail(&self) -> bool {
        self.position == self.contents.len().saturating_sub(1)
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
