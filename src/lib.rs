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

#[macro_use(defer)]
extern crate scopeguard;

/// A module providing the builder of [Prompt](struct.Prompt.html).
pub mod build {
    use crate::{Prompt, Result, Runnable};

    /// A trait to build [Prompt](struct.Prompt.html).
    pub trait Builder {
        fn build(self) -> Result<Prompt>;
        fn dispatcher(self) -> Result<Box<dyn Runnable>>;
    }
}

mod error {
    use std::io;

    /// Result for `prompt`.
    pub type Result<T> = std::result::Result<T, io::Error>;
}
pub use error::Result;

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
    /// A data structure to store the suggestions for the completion.
    pub mod selector;
}
/// A module providing the lines to receive and display user inputs.
pub mod readline {
    pub mod action;
    mod builder;
    mod event;
    mod keybind;
    mod state;

    pub use self::builder::Builder;
    pub use self::state::{Mode, State};
}

/// A module providing the selectbox to choose the items from.
pub mod select {
    pub mod action;
    mod builder;
    mod dispatcher;
    mod keybind;
    mod state;

    pub use self::builder::Builder;
    pub use self::state::State;
}
pub(crate) mod text {
    mod state;

    pub use self::state::State;
}

/// Collection of terminal operations.
pub mod cmd;
/// Characters and their width.
pub mod grapheme;
/// Register the pairs of
/// [crossterm event](../crossterm/event/enum.Event.html)
/// and their handlers.
pub mod keybind;
pub mod suggest;
/// Utilities for the terminal.
pub mod termutil;

use std::io;
use std::sync::Once;

pub use crossterm;

/// A type representing the actions when the events are received.
pub type Action<S> = dyn Fn(&mut io::Stdout, &mut S) -> Result<Option<String>>;

/// A core data structure to manage the hooks and state.
pub struct Prompt {
    out: io::Stdout,
    dispatcher: Box<dyn Runnable>,
}

pub trait Runnable {
    fn used_lines(&self) -> Result<u16>;
    fn handle_event(
        &mut self,
        _: &crossterm::event::Event,
        _: &mut io::Stdout,
    ) -> Result<Option<String>>;
    fn initialize(&mut self, _: &mut io::Stdout) -> Result<Option<String>>;
    fn finalize(&mut self, _: &mut io::Stdout) -> Result<Option<String>>;
    fn pre_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>>;
    fn post_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>>;
}

static ONCE: Once = Once::new();

impl Prompt {
    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<String> {
        ONCE.call_once(|| {
            termutil::clear(&mut self.out).ok();
        });

        crossterm::terminal::enable_raw_mode()?;
        defer! {{
            crossterm::terminal::disable_raw_mode().ok();
        }};

        if let Err(e) = self.dispatcher.initialize(&mut self.out) {
            self.dispatcher.finalize(&mut self.out)?;
            return Err(e);
        }

        loop {
            // check whether to be able to render.
            if crossterm::terminal::size()?.1 < self.dispatcher.used_lines()? {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Terminal does not leave the space to render.",
                ));
            }

            // hook pre_run
            if let Err(e) = self.dispatcher.pre_run(&mut self.out) {
                self.dispatcher.finalize(&mut self.out)?;
                return Err(e);
            }

            let ev = crossterm::event::read()?;
            match self.dispatcher.handle_event(&ev, &mut self.out) {
                Ok(maybe_ret) => {
                    if let Some(ret) = maybe_ret {
                        self.dispatcher.finalize(&mut self.out)?;
                        return Ok(ret);
                    }
                }
                Err(e) => {
                    self.dispatcher.finalize(&mut self.out)?;
                    return Err(e);
                }
            }

            // hook post_run
            if let Err(e) = self.dispatcher.post_run(&mut self.out) {
                self.dispatcher.finalize(&mut self.out)?;
                return Err(e);
            }
        }
    }
}
