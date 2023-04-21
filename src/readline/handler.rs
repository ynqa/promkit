use std::{io, marker::PhantomData};

use crate::{
    grapheme::Grapheme,
    internal::buffer::Buffer,
    readline::{Mode, State},
    termutil, InputHandler, Output, ResizeHandler, Result,
};

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
        ch: char,
        _out: &mut io::Stdout,
        state: &mut State,
    ) -> Result<Option<<State as Output>::Output>> {
        if let Some(limit) = state.buffer_limit()? {
            if limit <= state.editor.data.width() {
                return Ok(None);
            }
        }
        match state.edit_mode {
            Mode::Insert => state.editor.insert(Grapheme::from(ch)),
            Mode::Overwrite => state.editor.overwrite(Grapheme::from(ch)),
        }
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
        state.render_static(out)?;
        // Overwrite the prev as default.
        state.prev = Buffer::default();
        Ok(None)
    }
}
