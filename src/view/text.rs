use std::any::Any;

use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{matrixify, Graphemes},
    pane::Pane,
};

use super::{AsAny, Viewable};

#[derive(Clone)]
pub struct TextViewer {
    pub text: String,
    pub style: ContentStyle,
}

impl Viewable for TextViewer {
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

impl AsAny for TextViewer {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
