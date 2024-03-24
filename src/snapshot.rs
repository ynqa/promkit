use std::{
    any::{type_name, Any},
    cell::{Ref, RefCell},
};

use crate::{pane::Pane, AsAny, Error, Renderer, Result};

/// A `Snapshot` struct captures the state of a renderer at three different points:
/// initial (`init`), before any changes (`before`), and after changes have been applied (`after`).
/// It is generic over `R` where `R` must implement the `Renderer` and `Clone` traits.
#[derive(Clone)]
pub struct Snapshot<R: Renderer + Clone> {
    init: R,
    before: RefCell<R>,
    after: R,
}

impl<R: Renderer + Clone + 'static> Renderer for Snapshot<R> {
    /// Creates panes based on the `after` state of the renderer, and updates the `before` state
    /// to match the `after` state. This method is called to render the current state of the UI.
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        *self.before.borrow_mut() = self.after.clone();
        self.after.create_panes(width)
    }
}

// TODO: enable Snapshot<R> to use impl_as_any macro.
impl<R: Renderer + Clone + 'static> AsAny for Snapshot<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// TODO: enable Snapshot<R> to use impl_cast macro.
impl<R: Renderer + Clone + 'static> Snapshot<R> {
    pub fn cast_mut(renderer: &mut dyn crate::Renderer) -> Result<&mut Self> {
        let snapshot = renderer
            .as_any_mut()
            .downcast_mut::<Self>()
            .ok_or_else(|| Error::DowncastError(type_name::<Self>().to_string()))?;
        Ok(snapshot)
    }

    pub fn cast(renderer: &dyn crate::Renderer) -> Result<&Self> {
        let snapshot = renderer
            .as_any()
            .downcast_ref::<Self>()
            .ok_or_else(|| Error::DowncastError(type_name::<Self>().to_string()))?;
        Ok(snapshot)
    }
}

impl<R: Renderer + Clone + 'static> Snapshot<R> {
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
