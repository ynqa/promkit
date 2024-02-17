use std::{any::Any, cell::RefCell};

use crate::{crossterm::event::Event, pane::Pane};

/// A trait for objects that can be rendered in the terminal.
/// It requires the ability to create a pane, handle events, and perform cleanup.
pub trait Renderable: AsAny {
    /// Creates a pane with the given width.
    fn make_pane(&self, width: u16) -> Pane;
    /// Handles terminal events.
    fn handle_event(&mut self, event: &Event);
    /// Performs something (e.g. cleanup) after rendering is complete.
    fn postrun(&mut self);
}

/// A trait for casting objects to `Any`, allowing for dynamic typing.
pub trait AsAny {
    /// Returns `Any`.
    fn as_any(&self) -> &dyn Any;
}

/// A struct to manage the state of a `Renderable` object.
/// It keeps track of the initial, before, and after states of the object.
pub struct State<R: Renderable> {
    pub init: R,
    pub before: R,
    pub after: RefCell<R>,
}

impl<R: Renderable + Clone> State<R> {
    /// Creates a new `State` with the given `Renderable` object.
    pub fn new(renderable: R) -> Self {
        Self {
            init: renderable.clone(),
            before: renderable.clone(),
            after: RefCell::new(renderable),
        }
    }
}

impl<R: Clone + Renderable + 'static> Renderable for State<R> {
    /// Delegates the creation of a pane to the `after` state.
    fn make_pane(&self, width: u16) -> Pane {
        self.after.borrow().make_pane(width)
    }

    /// Updates the `before` state and delegates event handling to the `after` state.
    fn handle_event(&mut self, event: &Event) {
        self.before = self.after.borrow().clone();
        self.after.borrow_mut().handle_event(event);
    }

    /// Performs cleanup on the `after` state and updates the `init` and `before` states.
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
