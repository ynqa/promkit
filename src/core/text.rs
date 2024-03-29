use crate::{
    crossterm::style::ContentStyle,
    grapheme::{matrixify, StyledGraphemes},
    impl_as_any,
    pane::Pane,
};

/// A renderer for displaying text within a pane.
///
/// This struct is responsible for rendering text with a specific style
/// and handling events that are relevant to the rendered text.
#[derive(Clone)]
pub struct Renderer {
    /// The text to be rendered.
    pub text: String,

    /// Style for the text string.
    pub style: ContentStyle,
}

impl Renderer {
    pub fn replace(&mut self, renderer: Self) {
        *self = renderer;
    }
}

impl_as_any!(Renderer);

impl crate::Renderer for Renderer {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        vec![Pane::new(
            matrixify(
                width as usize,
                &StyledGraphemes::from_str(&self.text, self.style),
            ),
            0,
            None,
        )]
    }
}
