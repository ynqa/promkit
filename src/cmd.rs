use std::io::{Error, ErrorKind, Stdout};

use crate::{EventHandleFn, Output};

/// Leave from event-loop with exit code `0`.
pub fn enter<S: Output>() -> Box<EventHandleFn<S>> {
    Box::new(|_: &mut Stdout, state: &mut S| Ok(Some(state.output())))
}

/// Leave from event-loop with io::ErrorKind::Interrupted error.
pub fn interrupt<S: Output>() -> Box<EventHandleFn<S>> {
    Box::new(|_: &mut Stdout, _: &mut S| Err(Error::from(ErrorKind::Interrupted)))
}
