use std::{io, marker::PhantomData};

use crate::{
    internal::selector::Selector, select::State, termutil, InputHandler, Output, ResizeHandler,
    Result,
};

#[derive(Default)]
pub struct Handler<S> {
    _phantom: PhantomData<S>,
}

impl Default for Handler<State> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl InputHandler<State> for Handler<State> {
    fn handle(
        &mut self,
        _: char,
        _: &mut io::Stdout,
        _: &mut State,
    ) -> Result<Option<<State as Output>::Output>> {
        Ok(None)
    }
}

impl ResizeHandler<State> for Handler<State> {
    fn handle(
        &mut self,
        _: (u16, u16),
        out: &mut io::Stdout,
        state: &mut State,
    ) -> Result<Option<<State as Output>::Output>> {
        termutil::clear(out)?;
        state.editor.to_head();
        state.cursor.to_head();
        state.pre_render(out)?;
        // Overwrite the prev as default.
        state.prev = Selector::default();
        Ok(None)
    }
}
