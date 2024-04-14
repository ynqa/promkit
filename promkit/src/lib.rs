//! # promkit
//!
//! [![ci](https://github.com/ynqa/promkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ynqa/promkit/actions/workflows/ci.yml)
//! [![docs.rs](https://img.shields.io/docsrs/promkit)](https://docs.rs/promkit)
//!
//! A toolkit for building your own interactive prompt in Rust.
//!
//! ## Getting Started
//!
//! Put the package in your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! promkit = "0.4.0"
//! ```
//!
//! ## Features
//!
//! - Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
//! - Various building methods
//!   - Preset; Support for quickly setting up a UI by providing simple parameters.
//!     - [Readline](https://github.com/ynqa/promkit/tree/v0.4.0#readline)
//!     - [Confirm](https://github.com/ynqa/promkit/tree/v0.4.0#confirm)
//!     - [Password](https://github.com/ynqa/promkit/tree/v0.4.0#password)
//!     - [Select](https://github.com/ynqa/promkit/tree/v0.4.0#select)
//!     - [QuerySelect](https://github.com/ynqa/promkit/tree/v0.4.0#queryselect)
//!     - [Checkbox](https://github.com/ynqa/promkit/tree/v0.4.0#checkbox)
//!     - [Tree](https://github.com/ynqa/promkit/tree/v0.4.0#tree)
//!   - Combining various UI components.
//!     - They are provided with the same interface, allowing users to choose and
//!       assemble them according to their preferences.
//!   - (Upcoming) Stronger support to display yor own data structures.
//! - Versatile customization capabilities
//!   - Theme for designing the appearance of the prompt.
//!     - e.g. cursor, text
//!   - Validation for user input and error message construction.
//! - Mouse support (partially)
//!   - Allows scrolling through lists with the mouse wheel
//!
//! ## Examples/Demos
//!
//! See [here](https://github.com/ynqa/promkit/tree/v0.4.0#examplesdemos)
//!
//! ## Why *promkit*?
//!
//! Related libraries in this category include the following:
//! - [console-rs/dialoguer](https://github.com/console-rs/dialoguer)
//! - [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire)
//!
//! *promkit* offers several advantages over these libraries:
//!
//! ### Unified interface approach for UI components
//!
//! *promkit* takes a unified approach by having all of its components inherit the
//! same `Renderer` trait. This design choice enables users to seamlessly support
//! their custom data structures for display, similar to the relationships seen in
//! TUI projects like [ratatui-org/ratatui](https://github.com/ratatui-org/ratatui)
//! and
//! [EdJoPaTo/tui-rs-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget).
//! In other words, it's straightforward for anyone to display their own data
//! structures using widgets within promkit.  
//! In contrast, other libraries tend to treat each prompt as a mostly independent
//! entity. If you want to display a new data structure, you often have to build the
//! UI from scratch, which can be a time-consuming and less flexible process.
//!
//!   ```ignore
//!   pub trait Renderer {
//!       fn create_panes(&self, width: u16) -> Vec<Pane>;
//!   }
//!   ```
//!
//! ### Variety of Pre-built UI Preset Components
//!
//! One of the compelling reasons to choose *promkit* is its extensive range of pre-built UI preset components.
//! These presets allow developers to quickly implement various interactive prompts without the need to design and
//! build each component from scratch. The availability of these presets not only speeds up the development process
//! but also ensures consistency and reliability across different applications.
//! Here are some of the preset components available, see [Examples](#examplesdemos)
//!
//! ### Resilience to terminal resizing
//!
//! Performing operations that involve executing a command in one pane while
//! simultaneously opening a new pane is a common occurrence. During such operations,
//! if UI corruption is caused by resizing the terminal size, it may adversely affect
//! the user experience.  
//! Other libraries can struggle when the terminal is resized, making typing and
//! interaction difficult or impossible. For example:
//!
//!  - [(console-rs/dialoguer) Automatic re-render on terminal window resize](https://github.com/console-rs/dialoguer/issues/178)
//!
//! *promkit* introduces a step to align data with the screen size before rendering.
//! This approach ensures consistency in UI elements even when
//! the terminal size changes, providing a smoother user experience.

pub use crossterm;
pub use serde_json;

mod core;
pub use core::*;
mod error;
pub use error::{Error, Result};
pub mod grapheme;
mod macros;
pub mod pane;
pub mod preset;
pub mod style;
pub mod suggest;
pub mod switch;
pub mod terminal;
pub mod validate;

use std::{any::Any, io};

use crate::{
    crossterm::{
        cursor,
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    pane::Pane,
    terminal::Terminal,
};

/// Represents the signal to control the flow of a prompt.
///
/// This enum is used to indicate whether a prompt should continue running
/// or quit based on user input or other conditions.
#[derive(Eq, PartialEq)]
pub enum PromptSignal {
    /// Indicates that the prompt should continue to run and handle further events.
    Continue,
    /// Indicates that the prompt should quit and terminate its execution.
    Quit,
}

/// Type definition for an event handler function.
///
/// This function type is used to handle events within a prompt. It takes a reference to an `Event`
/// and a mutable reference to a state of type `S`, and returns a `Result` containing a `PromptSignal`.
/// The `PromptSignal` indicates whether the prompt should continue running or quit.
///
/// # Arguments
///
/// * `event` - A reference to the event that occurred.
/// * `state` - A mutable reference to the state `S` of the prompt, allowing the handler to modify it.
///
/// # Returns
///
/// Returns a `Result` with a `PromptSignal`, indicating the next action for the prompt.
pub type EventHandler<S> = fn(&Event, &mut S) -> Result<PromptSignal>;

pub trait Renderer: AsAny {
    /// Creates panes with the given width.
    fn create_panes(&self, width: u16) -> Vec<Pane>;
}

/// A trait for casting objects to `Any`, allowing for dynamic typing.
pub trait AsAny {
    /// Returns `Any`.
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Type alias for a dynamic evaluator function.
///
/// This evaluator function is responsible for handling events during the prompt's execution.
/// It takes a reference to an `Event` and a mutable reference to a boxed `Renderer` instance,
/// and returns a `Result` containing a `PromptSignal`. The `PromptSignal` indicates whether
/// the prompt should continue running or quit.
///
/// # Arguments
///
/// * `event` - A reference to the event that occurred.
/// * `renderer` - A mutable reference to a boxed instance of a type that implements the `Renderer` trait.
///
/// # Returns
///
/// Returns a `Result` with a `PromptSignal`, indicating the next action for the prompt.
pub type DynEvaluator = dyn Fn(&Event, &mut Box<dyn Renderer>) -> Result<PromptSignal>;

/// Type alias for a result producer function.
///
/// This function is used to produce the final result of the prompt based on the state of the renderer.
/// It takes a reference to a type that implements the `Renderer` trait and returns a `Result` containing
/// the final output of the prompt.
///
/// # Arguments
///
/// * `renderer` - A reference to an instance of a type that implements the `Renderer` trait.
///
/// # Returns
///
/// Returns a `Result` containing the final output of the prompt.
pub type ResultProducer<T> = fn(&dyn Renderer) -> Result<T>;

/// Represents a customizable prompt that can handle user input and produce a result.
///
/// This struct encapsulates the rendering logic,
/// event handling, and result production for a prompt.
pub struct Prompt<T> {
    renderer: Box<dyn Renderer>,
    evaluator: Box<DynEvaluator>,
    producer: ResultProducer<T>,
}

impl<T> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(
            io::stdout(),
            cursor::Show,
            event::DisableMouseCapture,
            cursor::MoveToNextLine(1),
        )
        .ok();
        disable_raw_mode().ok();
    }
}

impl<T> Prompt<T> {
    /// Attempts to create a new `Prompt` instance.
    ///
    /// # Arguments
    ///
    /// * `renderer` - A boxed renderer for drawing the prompt.
    /// * `evaluator` - A boxed evaluator function to handle events.
    /// * `producer` - A function to produce the result based on the renderer's state.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `Prompt` instance or an error.
    pub fn try_new(
        renderer: Box<dyn Renderer>,
        evaluator: Box<DynEvaluator>,
        producer: ResultProducer<T>,
    ) -> Result<Self> {
        Ok(Self {
            renderer,
            evaluator,
            producer,
        })
    }

    /// Runs the prompt, handling events and producing a result.
    ///
    /// This method initializes the terminal, and enters a loop
    /// to handle events until a quit signal is received.
    /// After exiting the loop, it produces and returns the result.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the produced result or an error.
    pub fn run(&mut self) -> Result<T> {
        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        let size = crossterm::terminal::size()?;
        let panes = self.renderer.create_panes(size.0);
        let mut terminal = Terminal::start_session(&panes)?;
        terminal.draw(&panes)?;

        loop {
            let ev = event::read()?;

            match &ev {
                Event::Resize(_, _) => {
                    terminal.position = (0, 0);
                    crossterm::execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge),
                    )?;
                }
                _ => {
                    if (self.evaluator)(&ev, &mut self.renderer)? == PromptSignal::Quit {
                        break;
                    }
                }
            }

            let size = crossterm::terminal::size()?;
            terminal.draw(&self.renderer.create_panes(size.0))?;
        }

        (self.producer)(&*self.renderer)
    }
}
