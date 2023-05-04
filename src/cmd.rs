use std::fmt;
use std::io::{Error, ErrorKind, Stdout};

use crate::{grid::UpstreamContext, Action};

/// Leave from event-loop with exit code `0`.
pub fn enter<S: fmt::Display>() -> Box<Action<S>> {
    Box::new(|_: &mut Stdout, _: &UpstreamContext, state: &mut S| Ok(Some(format!("{}", state))))
}

/// Leave from event-loop with io::ErrorKind::Interrupted error.
pub fn interrupt<S>() -> Box<Action<S>> {
    Box::new(|_: &mut Stdout, _: &UpstreamContext, _: &mut S| {
        Err(Error::from(ErrorKind::Interrupted))
    })
}
