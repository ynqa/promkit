use crate::{crossterm::event::Event, pane::Pane};

mod builder;
pub use builder::{text::TextBuilder, text_editor::TextEditorBuilder};
mod text;
mod text_editor;
pub use text_editor::Mode;

pub trait Editor {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}
