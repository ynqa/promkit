use std::{any::Any, cell::RefCell};

use crate::{crossterm::event::Event, pane::Pane, Result};

/// Represents the action to be taken after an event is processed.
///
/// This enum is used to determine how the `Prompt::run` method should proceed
/// after handling an event for a `Renderable` component.
///
/// - `Continue`: Indicates that the prompt should continue running and process further events.
/// - `Quit`: Signals that the prompt should stop running. If any of the `Renderable` components
///   returns `Quit`, a flag is set to indicate that the prompt should terminate. This allows
///   for a graceful exit when the user has completed their interaction with the prompt or when
///   an exit condition is met.
#[derive(Eq, PartialEq)]
pub enum EventAction {
    Continue,
    Quit,
}

/// A trait for objects that can be rendered in the terminal.
/// It requires the ability to create a pane, handle events,
/// and perform cleanup.
pub trait Renderable: AsAny {
    /// Creates a pane with the given width.
    fn make_pane(&self, width: u16) -> Pane;
    /// Handles terminal events.
    fn handle_event(&mut self, event: &Event) -> Result<EventAction>;
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

    /// Updates the `before` state and delegates event handling
    /// to the `after` state.
    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        self.before = self.after.borrow().clone();
        self.after.borrow_mut().handle_event(event)
    }

    /// Performs cleanup on the `after` state
    /// and updates the `init` and `before` states.
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
