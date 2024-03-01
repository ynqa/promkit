use std::{any::Any, cell::RefCell};

use crate::{crossterm::event::Event, pane::Pane, AsAny, EventAction, Renderer, Result};

/// A `Snapshot` captures the state of a renderer at different stages:
/// initial, before an event, and after an event.
///
/// It is designed to facilitate the tracking and comparison of renderer states over time,
/// especially useful in interactive applications
/// where the state may change in response to user input or other events.
///
/// The `Snapshot` struct is generic over `R`,
/// where `R` must implement the `Renderer` trait. This allows it to be
/// used with any renderer implementation.
///
/// Fields:
/// - `init`: The initial state of the renderer.
/// - `before`: The state of the renderer before the last event was processed.
/// - `after`: The state of the renderer after the last event was processed,
/// wrapped in a `RefCell` to allow for mutable borrowing at runtime.
pub struct Snapshot<R: Renderer> {
    pub init: R,
    pub before: R,
    pub after: RefCell<R>,
}

impl<R: Renderer + Clone> Snapshot<R> {
    /// Creates a new `Snapshot` instance with all states
    /// (`init`, `before`, `after`) initialized to the provided renderer state.
    ///
    /// Parameters:
    /// - `renderer`: The initial state of the renderer to be captured.
    pub fn new(renderer: R) -> Self {
        Self {
            init: renderer.clone(),
            before: renderer.clone(),
            after: RefCell::new(renderer),
        }
    }
}

impl<R: Clone + Renderer + 'static> Renderer for Snapshot<R> {
    /// Generates a `Pane` based on the current state of the renderer
    /// after the last event was processed.
    fn make_pane(&self, width: u16) -> Pane {
        self.after.borrow().make_pane(width)
    }

    /// Updates the `before` state to match the `after` state,
    /// then processes an event and updates the `after` state accordingly.
    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        self.before = self.after.borrow().clone();
        self.after.borrow_mut().handle_event(event)
    }

    /// Finalizes the renderer state after all events have been processed,
    /// updating the `init` and `before` states to match the final `after` state.
    fn postrun(&mut self) {
        self.after.borrow_mut().postrun();
        self.init = self.after.borrow().clone();
        self.before = self.after.borrow().clone();
    }
}

impl<R: Renderer + 'static> AsAny for Snapshot<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
