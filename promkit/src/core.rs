pub mod checkbox;
mod cursor;
pub mod json;
pub mod listbox;
pub mod snapshot;
pub mod text;
pub mod text_editor;
pub mod tree;

use crate::Pane;

pub trait PaneFactory {
    /// Creates pane with the given width.
    fn create_pane(&self, width: u16, height: u16) -> Pane;
}
