pub use crossterm;

pub mod grapheme;
// TODO: reconciliation (detecting differences between old and new grapheme trees)
pub mod render;
pub mod terminal;

pub trait GraphemeFactory {
    /// Creates styled graphemes with the given width and height.
    fn create_graphemes(&self, width: u16, height: u16) -> grapheme::StyledGraphemes;
}
