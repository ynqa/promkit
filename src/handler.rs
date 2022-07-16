use std::io::{Error, ErrorKind, Stdout};

use crate::{
    state::{Render, State},
    termutil, EventHandleFn,
};

/// Leave from event-loop with exit code `0`.
pub fn enter<S>() -> Box<EventHandleFn<S>> {
    Box::new(|_, _, _: &mut Stdout, _: &mut S| Ok(Some(0)))
}

/// Leave from event-loop with io::ErrorKind::Interrupted error.
pub fn interrupt<S>() -> Box<EventHandleFn<S>> {
    Box::new(|_, _, _: &mut Stdout, _: &mut S| Err(Error::from(ErrorKind::Interrupted)))
}

/// Reload terminal.
pub fn reload<D: Clone + Default, S, W>() -> Box<EventHandleFn<State<D, S>>>
where
    State<D, S>: Render<Stdout>,
{
    Box::new(|_, _, out: &mut Stdout, state: &mut State<D, S>| {
        termutil::clear(out)?;
        state.pre_render(out)?;
        // Overwrite the prev as default.
        state.0.prev = Box::new(D::default());
        Ok(None)
    })
}
