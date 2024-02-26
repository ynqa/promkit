//! # promkit
//!
//! [![.github/workflows/promkit.yml](https://github.com/ynqa/promkit/actions/workflows/promkit.yml/badge.svg)](https://github.com/ynqa/promkit/actions/workflows/promkit.yml)
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
//! same `Renderable` trait. This design choice enables users to seamlessly support
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
//!   pub trait Renderable {
//!       fn make_pane(&self, width: u16) -> Pane;
//!       fn handle_event(&mut self, event: &Event);
//!       fn postrun(&mut self);
//!   }
//!   ```
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

mod core;
pub use core::*;
mod engine;
pub mod error;
mod grapheme;
mod pane;
pub mod preset;
pub mod render;
pub mod style;
mod terminal;
pub mod validate;

use std::io;
use std::sync::Once;

use crate::{
    crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    error::{Error, Result},
    render::Renderable,
    terminal::Terminal,
};

type Evaluate = dyn Fn(&Event, &Vec<Box<dyn Renderable>>) -> Result<bool>;
type Output<T> = dyn Fn(&Vec<Box<dyn Renderable>>) -> Result<T>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<T> {
    renderables: Vec<Box<dyn Renderable>>,
    evaluator: Box<Evaluate>,
    output: Box<Output<T>>,
}

static ONCE: Once = Once::new();

impl<T> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::MoveToNextLine(1)).ok();
        execute!(io::stdout(), cursor::Show).ok();
        disable_raw_mode().ok();
    }
}

impl<T> Prompt<T> {
    /// Creates a new `Prompt` instance
    /// with specified renderables, evaluator, and output functions.
    ///
    /// # Arguments
    ///
    /// * `renderables` - A vector of boxed objects implementing
    /// the `Renderable` trait.
    /// These are the UI components that will be rendered.
    /// * `evaluator` - A function that takes an event
    /// and the current state of renderables,
    /// returning a `Result<bool>` indicating
    /// whether the prompt is ready to produce an output.
    /// * `output` - A function that takes the current state of renderables
    /// and returns a `Result<T>`, where `T` is the type of the output
    /// produced by the prompt.
    ///
    /// # Returns
    ///
    /// Returns a `Result` wrapping a new `Prompt` instance
    /// if successful, or an error if the creation fails.
    pub fn try_new<E, O>(
        renderables: Vec<Box<dyn Renderable>>,
        evaluator: E,
        output: O,
    ) -> Result<Self>
    where
        E: Fn(&Event, &Vec<Box<dyn Renderable>>) -> Result<bool> + 'static,
        O: Fn(&Vec<Box<dyn Renderable>>) -> Result<T> + 'static,
    {
        Ok(Self {
            renderables,
            evaluator: Box::new(evaluator),
            output: Box::new(output),
        })
    }

    /// Runs the prompt, handling events and rendering UI components
    /// until an output is produced or an error occurs.
    ///
    /// This method initializes the terminal
    /// in raw mode, hides the cursor, and enters a loop to handle events.
    /// It continuously renders the UI components
    /// based on the current state and events until the evaluator function
    /// indicates that the prompt is ready to produce
    /// an output or an interrupt signal (e.g., Ctrl+C) is received.
    ///
    /// # Returns
    ///
    /// Returns a `Result<T>`, where `T` is the type of the output
    /// produced by the prompt, or an error if the prompt fails to run.
    pub fn run(&mut self) -> Result<T> {
        let mut engine = Engine::new(io::stdout());

        ONCE.call_once(|| {
            engine.clear().ok();
        });

        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        let mut terminal = Terminal::start_session(&mut engine)?;
        let size = engine.size()?;
        terminal.draw(
            &mut engine,
            self.renderables
                .iter()
                .map(|editor| editor.make_pane(size.0))
                .collect(),
        )?;

        loop {
            let ev = event::read()?;

            for editor in &mut self.renderables {
                editor.handle_event(&ev);
            }

            let finalizable = (self.evaluator)(&ev, &self.renderables)?;

            let size = engine.size()?;
            terminal.draw(
                &mut engine,
                self.renderables
                    .iter()
                    .map(|editor| editor.make_pane(size.0))
                    .collect(),
            )?;

            match &ev {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => {
                    if finalizable {
                        break;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => return Err(Error::Interrupted("ctrl+c".into())),
                _ => (),
            }
        }

        let ret = (self.output)(&self.renderables);
        self.renderables.iter_mut().for_each(|editor| {
            editor.postrun();
        });
        ret
    }
}
