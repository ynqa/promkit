mod buffer;
mod history;
mod selectbox;
mod suggest;

pub use buffer::Buffer;
pub use history::History;
pub use selectbox::SelectBox;
pub use suggest::Suggest;

use std::cell::Cell;

/// A core data structure to manage the editable object and the position of cursor.
#[derive(Debug, Clone, Default)]
pub struct Editor<D> {
    pub data: D,
    pub idx: Cell<usize>,
}

/// A trait representing the position of cursor in the object.
pub trait Cursor {
    /// Current position.
    fn pos(&self) -> usize;
    /// Move the position of cursor backward.
    fn prev(&self) -> bool;
    /// Move the position of cursor forward.
    fn next(&self) -> bool;
    /// Move the position of cursor to head.
    fn to_head(&self);
    /// Move the position of cursor to tail.
    fn to_tail(&self);
}

/// A trait to register the items.
pub trait Register<T> {
    fn register(&mut self, _: T);
    fn register_all<U: IntoIterator<Item = T>>(&mut self, items: U) {
        for (_, item) in items.into_iter().enumerate() {
            self.register(item)
        }
    }
}
