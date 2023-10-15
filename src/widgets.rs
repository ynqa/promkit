use std::any::Any;

use crate::{crossterm::event::Event, pane::Pane};

mod builder;
pub use builder::{
    item_picker::ItemPickerBuilder, text::TextBuilder, text_editor::TextEditorBuilder,
};
mod text_editor;
pub use text_editor::{Mode, TextEditor};
mod item_picker;
pub use item_picker::ItemPicker;
mod text;

pub trait Widget: AsAny {
    fn gen_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
