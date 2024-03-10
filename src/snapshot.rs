use std::{
    any::{type_name, Any},
    cell::{Ref, RefCell, RefMut},
};

use crate::{pane::Pane, AsAny, Error, Renderer, Result};

pub struct Snapshot<R: Renderer> {
    init: R,
    before: RefCell<R>,
    after: RefCell<R>,
}

impl<R: Clone + Renderer + 'static> Renderer for Snapshot<R> {
    fn create_panes(&self, width: u16) -> Vec<Pane> {
        self.after.borrow().create_panes(width)
    }

    /// Finalizes the renderer state after all events have been processed,
    /// updating the `init` and `before` states to match the final `after` state.
    fn postrun(&mut self) {
        self.after.borrow_mut().postrun();
        self.init = self.after.borrow().clone();
        self.before = RefCell::new(self.after.borrow().clone());
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
    pub fn new(renderer: R) -> Self {
        Self {
            init: renderer.clone(),
            before: RefCell::new(renderer.clone()),
            after: RefCell::new(renderer),
        }
    }

    /// Returns a reference to the initial state of the renderer.
    pub fn init(&self) -> &R {
        &self.init
    }

    /// Attempts to cast a boxed `Renderer` to a `Snapshot<R>`.
    pub fn cast(renderer: &dyn Renderer) -> Result<&Snapshot<R>> {
        let snapshot: &Snapshot<R> = renderer
            .as_any()
            .downcast_ref::<Snapshot<R>>()
            .ok_or_else(|| Error::TypeCastError(type_name::<R>().to_string()))?;
        Ok(snapshot)
    }

    /// Attempts to cast a boxed `Renderer` to a `Snapshot<R>` and borrows its `after` state.
    pub fn cast_and_borrow_after(renderer: &dyn Renderer) -> Result<Ref<R>> {
        let snapshot: &Snapshot<R> = renderer
            .as_any()
            .downcast_ref::<Snapshot<R>>()
            .ok_or_else(|| Error::TypeCastError(type_name::<R>().to_string()))?;
        Ok(snapshot.after.borrow())
    }

    pub fn borrow_before(&self) -> Ref<R> {
        self.before.borrow()
    }

    pub fn borrow_mut_before(&self) -> RefMut<R> {
        self.before.borrow_mut()
    }

    /// Borrows the `after` state of the `Snapshot`.
    pub fn borrow_after(&self) -> Ref<R> {
        self.after.borrow()
    }

    /// Borrows the `after` state of the `Snapshot` mutably.
    pub fn borrow_mut_after(&self) -> RefMut<R> {
        self.after.borrow_mut()
    }
}
