//! # promkit
//!
//! A toolkit for building your own interactive command-line tools in Rust,
//! utilizing [crossterm](https://github.com/crossterm-rs/crossterm).
//!
//! ## Getting Started
//!
//! Put the package in your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! promkit = "0.1.1"
//! ```
//!
//! ## Examples
//!
//! Readline:
//!
//! ```no_run
//! use promkit::{build::Builder, readline, Result};
//!
//! fn main() -> Result<()> {
//!     let mut p = readline::Builder::default().build()?;
//!     loop {
//!         let (line, exit_code) = p.run()?;
//!         if exit_code == 0 {
//!             println!("result: {:?}", line);
//!         } else {
//!             return Ok(());
//!         }
//!     }
//! }
//! ```
//!
//! Select:
//!
//! ```no_run
//! use crossterm::style;
//! use promkit::{
//!     build::Builder,
//!     edit::{Register, SelectBox},
//!     select, Result,
//! };
//!
//! fn main() -> Result<()> {
//!     let mut selectbox = Box::new(SelectBox::default());
//!     selectbox.register_all((0..100).map(|v| v.to_string()).collect::<Vec<String>>());
//!     let mut p = select::Builder::default()
//!         .title("Q: What number do you like?")
//!         .title_color(style::Color::DarkGreen)
//!         .selectbox(selectbox)
//!         .build()?;
//!     let (line, exit_code) = p.run()?;
//!     if exit_code == 0 {
//!         println!("result: {:?}", line)
//!     }
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate downcast_rs;
#[macro_use(defer)]
extern crate scopeguard;

/// A module provides the builder of [Prompt](struct.Prompt.html).
pub mod build;
/// Data structures to be edited by the user interactions on the terminal.
pub mod edit;
mod error;
/// Characters and their width.
pub mod grapheme;
/// Collection of terminal operations.
pub mod handler;
/// Register the pairs of
/// [crossterm event](../crossterm/event/enum.Event.html)
/// and their handlers.
pub mod keybind;
/// A module providing the lines to receive and display user inputs.
pub mod readline;
/// A module providing the selectbox to choose the items from.
pub mod select;
/// State of applications.
pub mod state;
/// Utilities for the terminal.
pub mod termutil;

pub use error::Result;

use std::cell::RefCell;
use std::fmt::Display;
use std::io;
use std::rc::Rc;
use std::sync::Once;

pub use crossterm;
use downcast_rs::Downcast;

/// A type representing exit code.
pub type ExitCode = i32;

/// A trait for handling the events.
pub trait Handler<S>: Downcast {
    /// Edit the state and show the items on stdout on receiving the events.
    fn handle(
        &mut self,
        _: crossterm::event::Event,
        _: &mut io::Stdout,
        _: &mut S,
    ) -> Result<Option<ExitCode>>;
}
impl_downcast!(Handler<S>);

/// A type representing the hooks that are called in the certain timings.
pub type HookFn<S> = dyn Fn(&mut io::Stdout, &mut S) -> Result<()>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<S> {
    pub out: io::Stdout,
    pub handler: Rc<RefCell<dyn Handler<S>>>,
    /// Call initially every epoch in event-loop of
    /// [Prompt.run](struct.Prompt.html#method.run).
    pub pre_run: Option<Box<HookFn<S>>>,
    /// Call once initially when
    /// [Prompt.run](struct.Prompt.html#method.run) is called.
    pub initialize: Option<Box<HookFn<S>>>,
    /// Call once finally when
    /// [Prompt.run](struct.Prompt.html#method.run) is called.
    pub finalize: Option<Box<HookFn<S>>>,
    pub state: Box<S>,
}

/// A type representing the event handlers.
pub type EventHandleFn<S> =
    dyn Fn(Option<(u16, u16)>, Option<char>, &mut io::Stdout, &mut S) -> Result<Option<ExitCode>>;

/// A trait representing the final results for return.
pub trait Output {
    /// Return data type for the edited item.
    type Output: Display;
    /// Return the edited item.
    fn output(&self) -> Self::Output;
}

static ONCE: Once = Once::new();

impl<S: 'static + Output> Prompt<S> {
    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<(S::Output, ExitCode)> {
        ONCE.call_once(|| {
            termutil::clear(&mut self.out).ok();
        });

        crossterm::terminal::enable_raw_mode()?;
        defer! {{
            crossterm::terminal::disable_raw_mode().ok();
        }};

        if let Some(initialize) = &self.initialize {
            if let Err(e) = initialize(&mut self.out, &mut self.state) {
                if let Some(finalize) = &self.finalize {
                    finalize(&mut self.out, &mut self.state)?;
                }
                return Err(e);
            }
        }

        loop {
            if let Some(pre_run) = &self.pre_run {
                if let Err(e) = pre_run(&mut self.out, &mut self.state) {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
            }
            let ev = crossterm::event::read()?;
            match self
                .handler
                .borrow_mut()
                .handle(ev, &mut self.out, &mut self.state)
            {
                Ok(Some(exit_code)) => {
                    let item = self.state.output();
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Ok((item, exit_code));
                }
                Err(e) => {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
                _ => (),
            }
        }
    }
}
