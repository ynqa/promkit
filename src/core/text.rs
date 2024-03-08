use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{matrixify, StyledGraphemes},
    pane::Pane,
    AsAny, EventAction, Result,
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

impl crate::Renderer for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        Pane::new(
            matrixify(
                width as usize,
                &StyledGraphemes::from_str(&self.text, self.style),
            ),
            0,
            None,
        )
    }

    fn handle_event(&mut self, _event: &Event) -> Result<EventAction> {
        Ok(EventAction::Continue)
    }

    fn postrun(&mut self) {}
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
