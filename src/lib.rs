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
//!         let line = p.run()?;
//!         println!("result: {:?}", line);
//!     }
//! }
//! ```
//!
//! Select:
//!
//! ```no_run
//! use promkit::{
//!     build::Builder, crossterm::style, register::Register, select, selectbox::SelectBox, Result,
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
//!     let line = p.run()?;
//!     println!("result: {:?}", line);
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate downcast_rs;
#[macro_use(defer)]
extern crate scopeguard;

/// String buffer representing the user inputs.
pub mod buffer;
/// A module providing the builder of [Prompt](struct.Prompt.html).
pub mod build {
    use super::{state::State, Prompt, Result};

    /// A trait to build [Prompt](struct.Prompt.html).
    pub trait Builder<D, S> {
        fn state(self) -> Result<Box<State<D, S>>>;
        fn build(self) -> Result<Prompt<State<D, S>>>;
    }
}
/// Characters and their width.
pub mod grapheme;
/// Collection of terminal operations.
pub mod handler;
/// A data structure to store the history of the user inputs.
pub mod history;
/// Register the pairs of
/// [crossterm event](../crossterm/event/enum.Event.html)
/// and their handlers.
pub mod keybind;
/// A module providing the lines to receive and display user inputs.
pub mod readline {
    pub mod handler;
    mod keybind;
    mod prompt;
    pub mod state;

    pub use self::prompt::Builder;
    pub use self::state::{Mode, State};
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
/// A module providing the selectbox to choose the items from.
pub mod select {
    pub mod handler;
    mod keybind;
    mod prompt;
    pub mod state;

    pub use self::prompt::Builder;
    pub use self::state::State;
}
/// List representing the candidate items to be chosen by the users.
pub mod selectbox;
/// State of applications.
pub mod state {
    use std::io;

    use crate::Result;

    #[derive(Default)]
    pub struct State<D, S>(pub Inherited<D>, pub S);

    #[derive(Default)]
    pub struct Inherited<D> {
        pub prev: Box<D>,
        pub next: Box<D>,
        pub editor: Box<D>,
    }

    /// A trait to render the items into the output stream.
    pub trait Render<W: io::Write> {
        fn pre_render(&self, out: &mut W) -> Result<()>;
        fn render(&mut self, out: &mut W) -> Result<()>;
    }
}
/// A data structure to store the suggestions for the completion.
pub mod suggest;
/// Utilities for the terminal.
pub mod termutil;

mod error;
pub use error::Result;

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
    dyn Fn(Option<(u16, u16)>, Option<char>, &mut io::Stdout, &mut S) -> Result<bool>;

/// A trait representing the final results for return.
pub trait Output {
    /// Return data type for the edited item.
    type Output: Display;
    /// Return the edited item.
    fn output(&self) -> Self::Output;
}

static ONCE: Once = Once::new();

impl<D: 'static + Clone, S: 'static> Prompt<state::State<D, S>>
where
    state::State<D, S>: Output,
{
    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<<state::State<D, S> as Output>::Output> {
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
            self.state.0.prev = self.state.0.editor.clone();
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
            self.state.0.next = self.state.0.editor.clone();
        }
    }
}
