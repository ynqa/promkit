use std::{
    any::{type_name, Any},
    cell::{Ref, RefCell, RefMut},
};

use crate::{crossterm::event::Event, pane::Pane, AsAny, Error, EventAction, Renderer, Result};

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
    init: R,
    before: R,
    after: RefCell<R>,
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<R: Renderer + Clone + 'static> Snapshot<R> {
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

    /// Returns a reference to the initial state of the renderer.
    pub fn init(&self) -> &R {
        &self.init
    }

    /// Attempts to cast a boxed `Renderer` to a `Snapshot<R>`.
    pub fn cast(renderer: &Box<dyn Renderer>) -> Result<&Snapshot<R>> {
        let snapshot: &Snapshot<R> = renderer
            .as_any()
            .downcast_ref::<Snapshot<R>>()
            .ok_or_else(|| Error::TypeCastError(type_name::<R>().to_string()))?;
        Ok(snapshot)
    }

    /// Borrows the `after` state of the `Snapshot`.
    pub fn borrow_after(&self) -> Ref<R> {
        self.after.borrow()
    }

    /// Borrows the `after` state of the `Snapshot` mutably.
    pub fn borrow_mut_after(&self) -> RefMut<R> {
        self.after.borrow_mut()
    }

    /// Compares the `before` and `after` states of the `Snapshot` using a provided comparison function.
    pub fn compare_states<F>(&self, compare: F) -> bool
    where
        F: FnOnce(&R, &R) -> bool,
    {
        compare(&self.before, &self.after.borrow())
    }

    /// Attempts to cast a boxed `Renderer` to a `Snapshot<R>` and borrows its `after` state.
    pub fn cast_and_borrow_after(renderer: &dyn Renderer) -> Result<Ref<R>> {
        let snapshot: &Snapshot<R> = renderer
            .as_any()
            .downcast_ref::<Snapshot<R>>()
            .ok_or_else(|| Error::TypeCastError(type_name::<R>().to_string()))?;
        Ok(snapshot.after.borrow())
    }
}
