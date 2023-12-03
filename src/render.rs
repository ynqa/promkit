use std::{any::Any, cell::RefCell};

use crate::{crossterm::event::Event, pane::Pane};

pub trait Renderable: AsAny {
    fn make_pane(&self, width: u16) -> Pane;
    fn handle_event(&mut self, event: &Event);
    fn postrun(&mut self);
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub struct State<R: Renderable> {
    pub init: R,
    pub before: R,
    pub after: RefCell<R>,
}

impl<R: Renderable + Clone> State<R> {
    pub fn new(renderable: R) -> Self {
        Self {
            init: renderable.clone(),
            before: renderable.clone(),
            after: RefCell::new(renderable),
        }
    }
}

impl<R: Clone + Renderable + 'static> Renderable for State<R> {
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
}

impl<R: Renderable + 'static> AsAny for State<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
