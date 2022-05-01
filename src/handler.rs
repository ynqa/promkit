use std::io::{Error, ErrorKind, Stdout};

use crate::EventHandleFn;

/// Leave from event-loop with exit code `0`.
pub fn enter<S>() -> Box<EventHandleFn<S>> {
    Box::new(|_, _, _: &mut Stdout, _: &mut S| Ok(Some(0)))
}

/// Leave from event-loop with io::ErrorKind::Interrupted error.
pub fn interrupt<S>() -> Box<EventHandleFn<S>> {
    Box::new(|_, _, _: &mut Stdout, _: &mut S| Err(Error::from(ErrorKind::Interrupted)))
}
