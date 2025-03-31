pub use crossterm;

pub mod grapheme;
pub mod pane;
pub use pane::Pane;
pub mod terminal;

pub trait PaneFactory {
    /// Creates pane with the given width.
    fn create_pane(&self, width: u16, height: u16) -> Pane;
}
