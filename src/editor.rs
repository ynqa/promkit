use crate::{crossterm::event::Event, pane::Pane};

pub mod builder;
pub use builder::text_editor::TextEditorBuilder;
mod text_editor;

pub trait Editor {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn reset(&mut self);
    fn to_string(&self) -> String;
}
