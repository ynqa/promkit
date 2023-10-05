use crate::{crossterm::event::Event, grapheme::Graphemes, pane::Pane};

use super::Editor;

pub struct Text {
    pub text: Graphemes,
}

impl Editor for Text {
    fn gen_pane(&self, width: u16) -> Pane {
        let mut buf = vec![];
        buf.append(&mut self.text.clone());

        let mut layout = vec![];
        let mut row = Graphemes::default();
        for ch in buf.iter() {
            let width_with_next_char = row.iter().fold(0, |mut layout, g| {
                layout += g.width;
                layout
            }) + ch.width;
            if !row.is_empty() && (width as usize) < width_with_next_char {
                layout.push(row);
                row = Graphemes::default();
            }
            if (width as usize) >= ch.width {
                row.push(ch.clone());
            }
        }
        layout.push(row);
        Pane { layout, offset: 0 }
    }

    fn handle_event(&mut self, _event: &Event) {}

    fn reset(&mut self) {}

    fn to_string(&self) -> String {
        self.text.to_string()
    }
}
