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
pub use text::Text;

pub trait Component: AsAny {
    fn make_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
    fn output(&self) -> String;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub struct State<C: Component> {
    pub init: C,
    pub before: C,
    pub after: RefCell<C>,
}

impl<C: Component + Clone> State<C> {
    pub fn new(component: C) -> Self {
        Self {
            init: component.clone(),
            before: component.clone(),
            after: RefCell::new(component),
        }
    }
}

impl<C: Clone + Component + 'static> Component for State<C> {
    fn make_pane(&self, width: u16) -> Pane {
        self.after.borrow().make_pane(width)
    }

    fn handle_event(&mut self, event: &Event) {
        self.before = self.after.borrow().clone();
        self.after.borrow_mut().handle_event(event);
    }

    fn postrun(&mut self) {
        self.after.borrow_mut().postrun();
        self.init = self.after.borrow().clone();
        self.before = self.after.borrow().clone();
    }

    fn output(&self) -> String {
        self.after.borrow().output()
    }
}

impl<C: Component + 'static> AsAny for State<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
