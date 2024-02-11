pub mod checkbox;
mod cursor;
pub mod listbox;
pub mod text;
pub mod text_editor;
pub mod tree;

pub trait Len {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl Len for String {
    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
