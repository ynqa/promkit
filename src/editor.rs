use crate::{crossterm::event::Event, pane::Pane};

pub mod text;

pub trait Editor {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn reset(&mut self);
    fn to_string(&self) -> String;
}
