use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{matrixify, StyledGraphemes},
    pane::Pane,
    AsAny, EventAction, Renderable, Result,
};

#[derive(Clone)]
pub struct Renderer {
    pub text: String,

    /// Style for text string.
    pub style: ContentStyle,
}

impl Renderable for Renderer {
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
}
