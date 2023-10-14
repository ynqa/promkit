use crate::{crossterm::event::Event, pane::Pane};

mod builder;
pub use builder::{readline::ReadlineBuilder, select::SelectBuilder, text::TextBuilder};
mod readline;
mod select;
pub use readline::Mode;
mod text;

pub trait Editor {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}
