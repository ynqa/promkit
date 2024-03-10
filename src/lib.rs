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
//! promkit = "0.3.0"
//! ```
//!
//! ## Features
//!
//! - Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
//! - Various building methods
//!   - Preset; Support for quickly setting up a UI by providing simple parameters.
//!     - [Readline](https://github.com/ynqa/promkit/tree/v0.3.0#readline)
//!     - [Confirm](https://github.com/ynqa/promkit/tree/v0.3.0#confirm)
//!     - [Password](https://github.com/ynqa/promkit/tree/v0.3.0#password)
//!     - [Select](https://github.com/ynqa/promkit/tree/v0.3.0#select)
//!     - [QuerySelect](https://github.com/ynqa/promkit/tree/v0.3.0#queryselect)
//!     - [Checkbox](https://github.com/ynqa/promkit/tree/v0.3.0#checkbox)
//!     - [Tree](https://github.com/ynqa/promkit/tree/v0.3.0#tree)
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
//! See [here](https://github.com/ynqa/promkit/tree/v0.3.0#examplesdemos)
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
//!       fn make_pane(&self, width: u16) -> Pane;
//!       fn handle_event(&mut self, event: &Event);
//!       fn postrun(&mut self);
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
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//! See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
//! file for details.
//!
//! ## Stargazers over time
//! [![Stargazers over time](https://starchart.cc/ynqa/promkit.svg?variant=adaptive)](https://starchart.cc/ynqa/promkit)

pub use crossterm;
pub use serde_json;

mod core;
pub use core::*;
mod engine;
pub mod error;
pub mod fake_input;
mod grapheme;
pub mod keymap;
mod pane;
pub mod preset;
pub mod snapshot;
pub mod style;
pub mod suggest;
mod terminal;
pub mod validate;

use std::{any::Any, io, sync::Once};

use crate::{
    crossterm::{
        cursor,
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    error::{Error, Result},
    pane::Pane,
    terminal::Terminal,
};

#[derive(Eq, PartialEq)]
pub enum PromptSignal {
    Continue,
    Quit,
}

pub type EventHandler<S> = fn(&mut S, &Event) -> Result<PromptSignal>;

pub trait Renderer: AsAny {
    /// Creates a pane with the given width.
    fn create_panes(&self, width: u16) -> Vec<Pane>;
    /// Performs something (e.g. cleanup) after rendering is complete.
    fn postrun(&mut self);
}

/// A trait for casting objects to `Any`, allowing for dynamic typing.
pub trait AsAny {
    /// Returns `Any`.
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

type DynEvaluator = dyn Fn(&Event, &mut Box<dyn Renderer>) -> Result<PromptSignal>;
type ResultProducer<T> = fn(&Box<dyn Renderer>) -> Result<T>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<T> {
    renderer: Box<dyn Renderer>,
    evaluator: Box<DynEvaluator>,
    producer: ResultProducer<T>,
    capture_mouse_events: bool,
}

static ONCE: Once = Once::new();

impl<T> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::MoveToNextLine(1)).ok();
        execute!(io::stdout(), cursor::Show).ok();
        execute!(io::stdout(), event::DisableMouseCapture).ok();
        disable_raw_mode().ok();
    }
}

impl<T> Prompt<T> {
    pub fn try_new(
        renderer: Box<dyn Renderer>,
        evaluator: Box<DynEvaluator>,
        producer: ResultProducer<T>,
        capture_mouse_events: bool,
    ) -> Result<Self> {
        Ok(Self {
            renderer,
            evaluator,
            producer,
            capture_mouse_events,
        })
    }

    pub fn run(&mut self) -> Result<T> {
        let mut engine = Engine::new(io::stdout());

        ONCE.call_once(|| {
            engine.clear().ok();
        });

        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;
        if self.capture_mouse_events {
            execute!(io::stdout(), event::EnableMouseCapture)?;
        }

        let mut terminal = Terminal::start_session(&mut engine)?;
        let size = engine.size()?;
        terminal.draw(&mut engine, self.renderer.create_panes(size.0))?;

        loop {
            let ev = event::read()?;

            if (self.evaluator)(&ev, &mut self.renderer)? == PromptSignal::Quit {
                break;
            }

            let size = engine.size()?;
            terminal.draw(&mut engine, self.renderer.create_panes(size.0))?;
        }

        let ret = (self.producer)(&self.renderer);
        self.renderer.postrun();
        ret
    }
}
