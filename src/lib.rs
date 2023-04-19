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
//! version = "0.1.2"
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
//!         let line = p.run()?;
//!         println!("result: {:?}", line);
//!     }
//! }
//! ```
//!
//! Select:
//!
//! ```no_run
//! use promkit::{build::Builder, crossterm::style, select, Result};
//!
//! fn main() -> Result<()> {
//!     let mut p = select::Builder::new(0..100)
//!         .title("Q: What number do you like?")
//!         .title_color(style::Color::DarkGreen)
//!         .build()?;
//!     let line = p.run()?;
//!     println!("result: {:?}", line);
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate downcast_rs;
#[macro_use(defer)]
extern crate scopeguard;

/// A module providing the builder of [Prompt](struct.Prompt.html).
pub mod build {
    use super::{Prompt, Result};

    /// A trait to build [Prompt](struct.Prompt.html).
    pub trait Builder<S> {
        fn build(self) -> Result<Prompt<S>>;
    }
}

/// A module providing trait to register the item into.
pub mod register {
    /// A trait to register the items.
    pub trait Register<T> {
        fn register(&mut self, _: T);
        fn register_all<U: IntoIterator<Item = T>>(&mut self, items: U) {
            for (_, item) in items.into_iter().enumerate() {
                self.register(item)
            }
        }
    }
}

pub(crate) mod internal {
    /// String buffer representing the user inputs.
    pub mod buffer;
    /// A data structure to store the history of the user inputs.
    pub mod history;
    /// A data structure to store the suggestions for the completion.
    pub mod selector;
}
/// A module providing the lines to receive and display user inputs.
pub mod readline {
    pub mod handler;
    mod keybind;
    mod prompt;
    pub mod state;

    pub use self::prompt::Builder;
    pub use self::state::{Mode, State};
}
/// A module providing the selectbox to choose the items from.
pub mod select {
    pub mod handler;
    mod keybind;
    mod prompt;
    pub mod state;

    pub use self::prompt::Builder;
    pub use self::state::State;
}

mod error;
pub use error::Result;
/// Characters and their width.
pub mod grapheme;
/// Collection of terminal operations.
pub mod handler;
/// Register the pairs of
/// [crossterm event](../crossterm/event/enum.Event.html)
/// and their handlers.
pub mod keybind;

pub mod suggest;
/// Utilities for the terminal.
pub mod termutil;

use std::cell::RefCell;
use std::fmt::Display;
use std::io;
use std::rc::Rc;
use std::sync::Once;

pub use crossterm;
use downcast_rs::Downcast;

/// A trait for handling the events.
pub trait Handler<S>: Downcast {
    /// Edit the state and show the items on stdout on receiving the events.
    fn handle(&mut self, _: crossterm::event::Event, _: &mut io::Stdout, _: &mut S)
        -> Result<bool>;
}
impl_downcast!(Handler<S>);

/// A type representing the hooks that are called in the certain timings.
pub type HookFn<S> = dyn Fn(&mut io::Stdout, &mut S) -> Result<()>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<S> {
    pub out: io::Stdout,
    pub state: S,
    pub handler: Rc<RefCell<dyn Handler<S>>>,
    /// Call initially every epoch in event-loop of
    /// [Prompt.run](struct.Prompt.html#method.run).
    pub pre_run: Option<Box<HookFn<S>>>,
    /// Call finally every epoch in event-loop of
    /// [Prompt.run](struct.Prompt.html#method.run).
    pub post_run: Option<Box<HookFn<S>>>,
    /// Call once initially when
    /// [Prompt.run](struct.Prompt.html#method.run) is called.
    pub initialize: Option<Box<HookFn<S>>>,
    /// Call once finally when
    /// [Prompt.run](struct.Prompt.html#method.run) is called.
    pub finalize: Option<Box<HookFn<S>>>,
}

/// A type representing the event handlers.
pub type EventHandleFn<S> =
    dyn Fn(Option<(u16, u16)>, Option<char>, &mut io::Stdout, &mut S) -> Result<bool>;

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
    pub fn run(&mut self) -> Result<S::Output> {
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
                Ok(true) => {
                    let item = self.state.output();
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Ok(item);
                }
                Err(e) => {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
                _ => (),
            }
            if let Some(post_run) = &self.post_run {
                if let Err(e) = post_run(&mut self.out, &mut self.state) {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
            }
        }
    }
}
