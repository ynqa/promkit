use crate::{
    crossterm::{event::Event, style::ContentStyle},
    grapheme::{matrixify, Graphemes},
    pane::Pane,
};

use super::Editor;

pub struct Text {
    pub text: String,
    pub style: ContentStyle,
}

impl Editor for Text {
    fn gen_pane(&self, width: u16) -> Pane {
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

    fn output(&self) -> String {
        self.text.to_string()
    }
}
