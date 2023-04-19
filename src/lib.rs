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
    use super::{InputHandler, Output, Prompt, ResizeHandler, Result};

    /// A trait to build [Prompt](struct.Prompt.html).
    pub trait Builder<S: Output, I: InputHandler<S>, R: ResizeHandler<S>> {
        fn build(self) -> Result<Prompt<S, I, R>>;
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
    /// A data structure to store the suggestions for the completion.
    pub mod selector;
}
/// A module providing the lines to receive and display user inputs.
pub mod readline {
    mod builder;
    pub mod cmd;
    mod keybind;
    mod state;

    pub use self::builder::Builder;
    pub use self::state::{Mode, State};
}
/// A module providing the selectbox to choose the items from.
pub mod select {
    mod builder;
    pub mod cmd;
    mod cursor;
    mod keybind;
    mod state;

    pub use self::builder::Builder;
    pub use self::state::State;
}

/// Collection of terminal operations.
pub mod cmd;
mod error;
pub use error::Result;
/// Characters and their width.
pub mod grapheme;
/// Register the pairs of
/// [crossterm event](../crossterm/event/enum.Event.html)
/// and their handlers.
pub mod keybind;
pub mod suggest;
/// Utilities for the terminal.
pub mod termutil;

use std::fmt::Display;
use std::io;
use std::sync::Once;

pub use crossterm;

/// A type representing the event handlers.
pub type EventHandleFn<S> =
    dyn Fn(&mut io::Stdout, &mut S) -> Result<Option<<S as Output>::Output>>;

pub trait InputHandler<S: Output> {
    fn handle(&mut self, _: char, _: &mut io::Stdout, _: &mut S) -> Result<Option<S::Output>>;
}

pub trait ResizeHandler<S: Output> {
    fn handle(&mut self, _: (u16, u16), _: &mut io::Stdout, _: &mut S)
        -> Result<Option<S::Output>>;
}

/// A type representing the hooks that are called in the certain timings.
pub type HookFn<S> = dyn Fn(&mut io::Stdout, &mut S) -> Result<()>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<S: Output, I: InputHandler<S>, R: ResizeHandler<S>> {
    pub out: io::Stdout,
    pub state: S,
    pub keybind: keybind::KeyBind<S>,
    pub input_handler: I,
    pub resize_handler: R,
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

/// A trait representing the final results for return.
pub trait Output {
    /// Return data type for the edited item.
    type Output: Display;
    /// Return the edited item.
    fn output(&self) -> Self::Output;
}

static ONCE: Once = Once::new();

impl<S: 'static + Output, I: InputHandler<S>, R: ResizeHandler<S>> Prompt<S, I, R> {
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
            // hook pre_run
            if let Some(pre_run) = &self.pre_run {
                if let Err(e) = pre_run(&mut self.out, &mut self.state) {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
            }
            let ev = crossterm::event::read()?;

            if let crossterm::event::Event::Resize(x, y) = ev {
                self.resize_handler
                    .handle((x, y), &mut self.out, &mut self.state)?;
            }
            match self.keybind.handle(&ev, &mut self.out, &mut self.state) {
                Ok(maybe_output) => {
                    if let Some(output) = maybe_output {
                        if let Some(finalize) = &self.finalize {
                            finalize(&mut self.out, &mut self.state)?;
                        }
                        return Ok(output);
                    }
                }
                Err(e) => {
                    if let Some(finalize) = &self.finalize {
                        finalize(&mut self.out, &mut self.state)?;
                    }
                    return Err(e);
                }
            }
            match ev {
                crossterm::event::Event::Key(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Char(ch),
                    modifiers: crossterm::event::KeyModifiers::NONE,
                    ..
                })
                | crossterm::event::Event::Key(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Char(ch),
                    modifiers: crossterm::event::KeyModifiers::SHIFT,
                    ..
                }) => {
                    self.input_handler
                        .handle(ch, &mut self.out, &mut self.state)?;
                }
                _ => (),
            }

            // hook post_run
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
