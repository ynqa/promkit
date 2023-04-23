use std::io::{Error, ErrorKind, Stdout};

use crate::{Action, Renderable};

/// Leave from event-loop with exit code `0`.
pub fn enter<R: Renderable>() -> Box<Action<R>> {
    Box::new(|_: &mut Stdout, state: &mut R| Ok(Some(state.output())))
}

/// Leave from event-loop with io::ErrorKind::Interrupted error.
pub fn interrupt<S>() -> Box<Action<S>> {
    Box::new(|_: &mut Stdout, _: &mut S| Err(Error::from(ErrorKind::Interrupted)))
}
