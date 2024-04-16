use std::{
    any::Any,
    cell::{Ref, RefCell},
};

use crate::{pane::Pane, AsAny, PaneFactory};

/// A `Snapshot` struct captures the state of a renderer at three different points:
/// initial (`init`), before any changes (`before`), and after changes have been applied (`after`).
/// It is generic over `R` where `R` must implement the `Renderer` and `Clone` traits.
#[derive(Clone)]
pub struct Snapshot<R: PaneFactory + Clone> {
    init: R,
    before: RefCell<R>,
    after: R,
}

impl<R: PaneFactory + Clone + 'static> PaneFactory for Snapshot<R> {
    fn create_pane(&self, width: u16, height: u16) -> Pane {
        *self.before.borrow_mut() = self.after.clone();
        self.after.create_pane(width, height)
    }
}

// TODO: enable Snapshot<R> to use impl_as_any macro.
impl<R: PaneFactory + Clone + 'static> AsAny for Snapshot<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<R: PaneFactory + Clone + 'static> Snapshot<R> {
    /// Constructs a new `Snapshot` instance with the given renderer. The `init`, `before`, and
    /// `after` states are all initialized with clones of the provided renderer.
    pub fn new(renderer: R) -> Self {
        Self {
            init: renderer.clone(),
            before: RefCell::new(renderer.clone()),
            after: renderer,
        }
    }

    /// Returns a reference to the initial state (`init`) of the renderer.
    pub fn init(&self) -> &R {
        &self.init
    }

    /// Returns a reference to the state of the renderer before any changes were applied (`before`).
    pub fn borrow_before(&self) -> Ref<R> {
        self.before.borrow()
    }

    /// Returns a reference to the state of the renderer after changes have been applied (`after`).
    pub fn after(&self) -> &R {
        &self.after
    }

    /// Returns a mutable reference to the state of the renderer after changes have been applied (`after`).
    /// This allows for modifications to the `after` state.
    pub fn after_mut(&mut self) -> &mut R {
        &mut self.after
    }

    /// Resets the `after` state to match the initial state (`init`). This method can be used to
    /// revert any changes made during the rendering cycle.
    pub fn reset_after_to_init(&mut self) {
        self.after = self.init.clone();
    }
}
