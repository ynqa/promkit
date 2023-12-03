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
//! promkit = "0.2.0"
//! ```
//!
//! ## Features
//!
//! - Support cross-platform both UNIX and Windows owing to [crossterm](https://github.com/crossterm-rs/crossterm)
//! - Various building methods
//!   - Support ranging from presets for easy use to layout building using `Viewable`s, and even for displaying your own data structures
//! - Versatile customization capabilities
//!   - Themes for defining the outer shell style, including text and cursor colors
//!   - Validation for user input and error message construction
//!   - and so on...
//!
//! ## Examples
//!
//! *promkit* provides presets so that users can utilize prompts immediately without
//! having to build complex components for specific use cases.  
//!
//! ```bash
//! cargo run --example readline
//! ```
//!
//! ![readline](https://github.com/ynqa/promkit/assets/6745370/afa75a49-f84b-444f-88e3-3dabca959164)
//!
//! The actual codes:
//!
//! ```ignore
//! use promkit::{error::Result, preset::Readline};
//!
//! fn main() -> Result {
//!     let mut p = Readline::default()
//!         .title("Feel free to fill in")
//!         .validator(
//!             |text| text.len() > 10,
//!             |text| format!("Length must be over 10 but got {}", text.len()),
//!         )
//!         .prompt()?;
//!     println!("result: {:?}", p.run()?);
//!     Ok(())
//! }
//! ```
//!
//! See [examples](https://github.com/ynqa/promkit/tree/main/examples/README.md)
//! for more examples.
//!
//! ## Why *promkit*?
//!
//! Similar libraries in this category include the following:
//! - [console-rs/dialoguer](https://github.com/console-rs/dialoguer)
//! - [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire)
//!
//! *promkit* offers several advantages over these libraries:
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
//! *promkit* processes the data to fit the screen size, reducing the likelihood of
//! rendering issues, such as misalignment. This approach ensures that UI elements
//! remain consistent even when the terminal is resized, providing a smoother user
//! experience.
//!
//! ### Unified component approach
//!
//! *promkit* takes a unified approach by having all of its components inherit the
//! same `Viewable` trait. This design choice enables users to seamlessly support
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
//!   pub trait Viewable {
//!       fn make_pane(&self, width: u16) -> Pane;
//!       fn handle_event(&mut self, event: &Event);
//!       fn postrun(&mut self);
//!   }
//!   ```
//!
//! In the provided presets of *promkit*, this mechanism is implemented. If you'd
//! like to try it out, you can refer to
//! the implementations of
//! [view](https://github.com/ynqa/promkit/tree/v0.2.0/src/view)
//! and
//! [preset](https://github.com/ynqa/promkit/tree/v0.2.0/src/preset)
//! for guidance.
//!
//! In summary, *promkit*'s resilience to terminal resizing and its unified component
//! approach make it a compelling choice for interactive command-line applications,
//! especially when compared to
//! [console-rs/dialoguer](https://github.com/console-rs/dialoguer) and
//! [mikaelmello/inquire](https://github.com/mikaelmello/inquire/tree/main/inquire).
//! These features provide a more reliable and extensible experience for developers,
//! allowing them to focus on building powerful command-line interfaces.
//!
//! ## Understanding dataflow and component interactions
//!
//! See [here](https://github.com/ynqa/promkit/tree/v0.2.0#understanding-dataflow-and-component-interactions)
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//! See the [LICENSE](https://github.com/ynqa/promkit/blob/main/LICENSE)
//! file for details.

extern crate scopeguard;

pub use crossterm;

mod engine;
pub mod error;
mod grapheme;
mod pane;
pub mod preset;
pub mod select_box;
pub mod style;
mod terminal;
pub mod text;
pub mod tree;
pub mod validate;
pub mod view;

use std::io;
use std::sync::Once;

use scopeguard::defer;

use crate::{
    crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    error::{Error, Result},
    terminal::Terminal,
    view::Viewable,
};

type Evaluate = dyn Fn(&Event, &Vec<Box<dyn Viewable>>) -> Result<bool>;
type Output<T> = dyn Fn(&Vec<Box<dyn Viewable>>) -> Result<T>;

/// A core data structure to manage the hooks and state.
pub struct Prompt<T> {
    viewables: Vec<Box<dyn Viewable>>,
    evaluator: Box<Evaluate>,
    output: Box<Output<T>>,
}

static ONCE: Once = Once::new();

impl<T> Prompt<T> {
    pub fn try_new<E, O>(viewables: Vec<Box<dyn Viewable>>, evaluator: E, output: O) -> Result<Self>
    where
        E: Fn(&Event, &Vec<Box<dyn Viewable>>) -> Result<bool> + 'static,
        O: Fn(&Vec<Box<dyn Viewable>>) -> Result<T> + 'static,
    {
        Ok(Self {
            viewables,
            evaluator: Box::new(evaluator),
            output: Box::new(output),
        })
    }

    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<T> {
        let mut engine = Engine::new(io::stdout());

        ONCE.call_once(|| {
            engine.clear().ok();
        });

        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;
        defer! {{
            execute!(io::stdout(), cursor::MoveToNextLine(1)).ok();
            execute!(io::stdout(), cursor::Show).ok();
            disable_raw_mode().ok();
        }};

        let mut terminal = Terminal::start_session(&mut engine)?;
        let size = engine.size()?;
        terminal.draw(
            &mut engine,
            self.viewables
                .iter()
                .map(|editor| editor.make_pane(size.0))
                .collect(),
        )?;

        loop {
            let ev = event::read()?;

            for editor in &mut self.viewables {
                editor.handle_event(&ev);
            }

            let finalizable = (self.evaluator)(&ev, &self.viewables)?;

            let size = engine.size()?;
            terminal.draw(
                &mut engine,
                self.viewables
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

        let ret = (self.output)(&self.viewables);
        self.viewables.iter_mut().for_each(|editor| {
            editor.postrun();
        });
        ret
    }
}
