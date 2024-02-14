/// Edit mode.
#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}
