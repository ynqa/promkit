use crate::{crossterm::event::Event, pane::Pane};

mod builder;
pub use builder::{item_picker::ItemPickerBuilder, readline::ReadlineBuilder, text::TextBuilder};
mod item_picker;
mod readline;
pub use readline::Mode;
mod text;

pub trait Editor {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}
