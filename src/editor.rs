use std::any::Any;

use crate::{crossterm::event::Event, pane::Pane};

mod builder;
pub use builder::{readline::ReadlineBuilder, select::SelectBuilder, text::TextBuilder};
mod readline;
pub use readline::{Mode, Readline};
mod select;
pub use select::Select;
mod text;

pub trait Editor: AsAny {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
