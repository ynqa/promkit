use std::{
    any::Any,
    cell::{Ref, RefCell},
};

use crate::{pane::Pane, AsAny, Renderer};

pub struct Snapshot<R: Renderer + Clone> {
    init: R,
    before: RefCell<R>,
    after: R,
}

impl<R: Renderer + Clone + 'static> Renderer for Snapshot<R> {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        *self.before.borrow_mut() = self.after.clone();
        self.after.create_panes(width)
    }

    /// Finalizes the renderer state after all events have been processed,
    /// updating the `init` and `before` states to match the final `after` state.
    fn postrun(&mut self) {
        self.after.postrun();
        self.init = self.after.clone();
        self.before = RefCell::new(self.after.clone());
    }
}

impl<R: Renderer + Clone + 'static> AsAny for Snapshot<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<R: Renderer + Clone + 'static> Snapshot<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            init: renderer.clone(),
            before: RefCell::new(renderer.clone()),
            after: renderer,
        }
    }

    /// Returns a reference to the initial state of the renderer.
    pub fn init(&self) -> &R {
        &self.init
    }

    pub fn borrow_before(&self) -> Ref<R> {
        self.before.borrow()
    }

    pub fn after(&self) -> &R {
        &self.after
    }

    pub fn after_mut(&mut self) -> &mut R {
        &mut self.after
    }

    pub fn reset_after_to_init(&mut self) {
        self.after = self.init.clone();
    }
}
