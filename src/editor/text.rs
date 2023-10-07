use crate::{
    crossterm::event::Event,
    grapheme::{matrixify, Graphemes},
    pane::Pane,
};

use super::Editor;

pub struct Text {
    pub text: Graphemes,
}

impl Editor for Text {
    fn gen_pane(&self, width: u16) -> Pane {
        let mut buf = Graphemes::default();
        buf.append(&mut self.text.clone());

        Pane::new(matrixify(width as usize, buf), 0)
    }

    fn handle_event(&mut self, _event: &Event) {}

    fn reset(&mut self) {}

    fn to_string(&self) -> String {
        self.text.text()
    }
}
