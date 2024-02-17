pub mod checkbox;
mod cursor;
pub mod listbox;
pub mod text;
pub mod text_editor;
pub mod tree;

/// Defines a `Len` trait for obtaining the length of a collection and checking if it is empty.
pub trait Len {
    /// Returns the length of the collection.
    fn len(&self) -> usize;

    /// Returns `true` if the collection is empty, otherwise `false`.
    fn is_empty(&self) -> bool;
}

/// Implements the `Len` trait for `Vec<T>`.
impl<T> Len for Vec<T> {
    /// Returns the number of elements in the vector.
    fn len(&self) -> usize {
        self.len()
    }

    /// Checks if the vector is empty.
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

/// Implements the `Len` trait for `String`.
impl Len for String {
    /// Returns the length of the string.
    fn len(&self) -> usize {
        self.len()
    }

    /// Checks if the string is empty.
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
