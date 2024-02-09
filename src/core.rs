mod cursor;
pub mod menu;
pub mod text;
pub mod text_editor;
pub mod tree;

pub trait Len {
    fn len(&self) -> usize;
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for String {
    fn len(&self) -> usize {
        self.len()
    }
}
