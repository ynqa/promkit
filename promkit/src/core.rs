pub mod checkbox;
mod cursor;
pub mod json;
pub mod listbox;
pub mod snapshot;
pub mod text;
pub mod text_editor;
pub mod tree;

use crate::{AsAny, Pane};

pub trait PaneFactory: AsAny {
    /// Creates pane with the given width.
    fn create_pane(&self, width: u16) -> Pane;
}
