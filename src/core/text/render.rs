use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    error::Result,
    grapheme::{matrixify, Graphemes},
    pane::Pane,
    render::{AsAny, Renderable, State},
};

#[derive(Clone)]
pub struct Renderer {
    pub text: String,

    /// Style for text string.
    pub style: ContentStyle,
}

impl State<Renderer> {
    pub fn try_new(text: String, style: ContentStyle) -> Result<Box<State<Renderer>>> {
        Ok(Box::new(State::<Renderer>::new(Renderer { text, style })))
    }
}

impl Renderable for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        Pane::new(
            matrixify(
                width as usize,
                &Graphemes::new_with_style(&self.text, self.style),
            ),
            0,
            None,
        )
    }

    fn handle_event(&mut self, _event: &Event) {}

    fn postrun(&mut self) {}
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
