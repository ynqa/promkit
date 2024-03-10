mod len;
use len::Len;
mod composite;
pub use composite::CompositeCursor;

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
            cyclic, // 追加
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

    /// Moves the cursor one position backward, if possible. Returns `true` if successful.
    /// If `cyclic` is true and the cursor is at the head, it moves to the tail.
    pub fn backward(&mut self) -> bool {
        if self.position > 0 {
            self.position = self.position.saturating_sub(1);
            true
        } else if self.cyclic && self.contents.len() > 0 {
            self.position = self.contents.len().saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// Moves the cursor one position forward, if possible. Returns `true` if successful.
    /// If `cyclic` is true and the cursor is at the tail, it moves to the head.
    pub fn forward(&mut self) -> bool {
        let l = self.contents.len();
        if l != 0 && self.position < l.saturating_sub(1) {
            self.position += 1;
            true
        } else if self.cyclic && l != 0 {
            self.position = 0;
            true
        } else {
            false
        }
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
            let mut b = Cursor::new(vec!["a", "b", "c"], 0, false);
            assert!(!b.backward());
            b.position = 1;
            assert!(b.backward());
        }
    }

    mod forward {
        use super::super::*;

        #[test]
        fn test() {
            let mut b = Cursor::new(vec!["a", "b", "c"], 0, false);
            assert!(b.forward());
            b.position = b.contents.len() - 1;
            assert!(!b.forward());
        }
    }
}
