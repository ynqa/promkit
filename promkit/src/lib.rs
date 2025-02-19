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
//! promkit = "0.6.1"
//! ```
//!
//! ## Features
//!
//! - Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
//! - Various building methods
//!   - Preset; Support for quickly setting up a UI by providing simple parameters.
//!     - [Readline](https://github.com/ynqa/promkit/tree/v0.6.1#readline)
//!     - [Confirm](https://github.com/ynqa/promkit/tree/v0.6.1#confirm)
//!     - [Password](https://github.com/ynqa/promkit/tree/v0.6.1#password)
//!     - [Select](https://github.com/ynqa/promkit/tree/v0.6.1#select)
//!     - [QuerySelect](https://github.com/ynqa/promkit/tree/v0.6.1#queryselect)
//!     - [Checkbox](https://github.com/ynqa/promkit/tree/v0.6.1#checkbox)
//!     - [Tree](https://github.com/ynqa/promkit/tree/v0.6.1#tree)
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
//! See [here](https://github.com/ynqa/promkit/tree/v0.6.1#examplesdemos)
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
//! ```ignore
//! pub trait Renderer: AsAny + Finalizer {
//!     /// Creates a collection of panes based on the specified width.
//!     ///
//!     /// This method is responsible for generating the layout of the UI components
//!     /// that will be displayed in the prompt. The width parameter allows the layout
//!     /// to adapt to the current terminal width.
//!     ///
//!     /// # Parameters
//!     ///
//!     /// * `width`: The width of the terminal in characters.
//!     ///
//!     /// # Returns
//!     ///
//!     /// Returns a vector of `Pane` objects that represent the layout of the UI components.
//!     fn create_panes(&self, width: u16) -> Vec<Pane>;
//!
//!     /// Evaluates an event and determines the next action for the prompt.
//!     ///
//!     /// This method is called whenever an event occurs (e.g., user input). It allows
//!     /// the renderer to react to the event and decide whether the prompt should continue
//!     /// running or quit.
//!     ///
//!     /// # Parameters
//!     ///
//!     /// * `event`: A reference to the event that occurred.
//!     ///
//!     /// # Returns
//!     ///
//!     /// Returns a `Result` containing a `PromptSignal`. `PromptSignal::Continue` indicates
//!     /// that the prompt should continue running, while `PromptSignal::Quit` indicates that
//!     /// the prompt should terminate its execution.
//!     fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal>;
//! }
//! ```
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
pub mod grapheme;
pub mod jsonz;
pub mod pane;
pub mod preset;
pub mod style;
pub mod suggest;
pub mod switch;
pub mod terminal;
pub mod validate;

use std::io;

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

pub trait Finalizer {
    /// The type of the result produced by the renderer.
    type Return;

    /// Finalizes the prompt and produces a result.
    ///
    /// This method is called after the prompt has been instructed to quit. It allows
    /// the renderer to perform any necessary cleanup and produce a final result.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the final result of the prompt. The type of the result
    /// is defined by the `Return` associated type.
    fn finalize(&mut self) -> anyhow::Result<Self::Return>;
}

/// A trait for rendering components within a prompt.
///
/// This trait defines the essential functions required for rendering custom UI components
/// in a prompt. Implementors of this trait can define how panes are created, how events
/// are evaluated, and how the final result is produced.
pub trait Renderer: Finalizer {
    /// Creates a collection of panes based on the specified width.
    ///
    /// This method is responsible for generating the layout of the UI components
    /// that will be displayed in the prompt. The width parameter allows the layout
    /// to adapt to the current terminal width and height.
    ///
    /// # Parameters
    ///
    /// * `width`: The width of the terminal in characters.
    /// * `height`: The height of the terminal in characters.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Pane` objects that represent the layout of the UI components.
    fn create_panes(&self, width: u16, height: u16) -> Vec<Pane>;

    /// Evaluates an event and determines the next action for the prompt.
    ///
    /// This method is called whenever an event occurs (e.g., user input). It allows
    /// the renderer to react to the event and decide whether the prompt should continue
    /// running or quit.
    ///
    /// # Parameters
    ///
    /// * `event`: A reference to the event that occurred.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PromptSignal`. `PromptSignal::Continue` indicates
    /// that the prompt should continue running, while `PromptSignal::Quit` indicates that
    /// the prompt should terminate its execution.
    fn evaluate(&mut self, event: &Event) -> anyhow::Result<PromptSignal>;
}

/// Represents a customizable prompt that can handle user input and produce a result.
///
/// This struct encapsulates the rendering logic,
/// event handling, and result production for a prompt.
pub struct Prompt<T: Renderer> {
    pub renderer: T,
    pub writer: Box<dyn io::Write>,
}

impl<T: Renderer> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(
            self.writer,
            cursor::Show,
            event::DisableMouseCapture,
            cursor::MoveToNextLine(1),
        )
        .ok();
        disable_raw_mode().ok();
    }
}

impl<T: Renderer> Prompt<T> {
    /// Runs the prompt, handling events and producing a result.
    ///
    /// This method initializes the terminal, and enters a loop
    /// to handle events until a quit signal is received.
    /// After exiting the loop, it produces and returns the result.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the produced result or an error.
    pub fn run(&mut self) -> anyhow::Result<T::Return> {
        enable_raw_mode()?;
        execute!(self.writer, cursor::Hide)?;

        let size = crossterm::terminal::size()?;
        let panes = self.renderer.create_panes(size.0, size.1);
        let mut terminal = Terminal::start_session(&panes, &mut self.writer)?;
        terminal.draw(&panes)?;

        loop {
            let ev = event::read()?;

            match &ev {
                Event::Resize(_, _) => {
                    terminal.on_resize()?;
                }
                _ => {
                    if self.renderer.evaluate(&ev)? == PromptSignal::Quit {
                        // Renderer has a possibility to disable the cursor color to indicate termination,
                        // and so ensure to display the state of Renderer at the end.
                        terminal.draw(&self.renderer.create_panes(size.0, size.1))?;
                        break;
                    }
                }
            }

            let size = crossterm::terminal::size()?;
            terminal.draw(&self.renderer.create_panes(size.0, size.1))?;
        }

        disable_raw_mode()?;
        self.renderer.finalize()
    }
}
