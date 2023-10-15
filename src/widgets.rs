use std::{any::Any, cell::RefCell};

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
}

pub struct State<W: Widget> {
    pub init: W,
    pub before: W,
    pub after: RefCell<W>,
}

impl<W: Clone + Widget + 'static> Widget for State<W> {
    fn gen_pane(&self, width: u16) -> Pane {
        self.after.borrow().gen_pane(width)
    }

    fn handle_event(&mut self, event: &Event) {
        self.before = self.after.borrow().clone();
        self.after.borrow_mut().handle_event(event);
    }

    fn postrun(&mut self) {
        self.before = self.init.clone();
        self.after = RefCell::new(self.init.clone());
    }

    fn output(&self) -> String {
        self.after.borrow().output()
    }
}

impl<W: Widget + 'static> AsAny for State<W> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
