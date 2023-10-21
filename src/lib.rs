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
//! ```ignore
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
//! ```ignore
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

extern crate scopeguard;

pub use crossterm;

pub mod components;
mod engine;
pub mod error;
mod grapheme;
mod history;
pub mod item_box;
mod pane;
pub mod preset;
pub mod style;
pub mod suggest;
mod terminal;
mod text_buffer;
mod theme;
mod validate;

use std::io;
use std::sync::Once;

use scopeguard::defer;

use crate::{
    components::Component,
    crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    error::{Error, Result},
    terminal::Terminal,
};

type Evaluate = dyn Fn(&Event, &Vec<Box<dyn Component>>) -> Result<bool>;

pub struct PromptBuilder {
    components: Vec<Box<dyn Component>>,
    evaluate: Option<Box<Evaluate>>,
}

impl PromptBuilder {
    pub fn new(components: Vec<Box<dyn Component>>) -> Self {
        Self {
            components,
            evaluate: None,
        }
    }

    pub fn evaluate<F: Fn(&Event, &Vec<Box<dyn Component>>) -> Result<bool> + 'static>(
        mut self,
        evaluate: F,
    ) -> Self {
        self.evaluate = Some(Box::new(evaluate));
        self
    }

    pub fn build(self) -> Result<Prompt> {
        Ok(Prompt {
            components: self.components,
            evaluate: self.evaluate,
        })
    }
}

/// A core data structure to manage the hooks and state.
pub struct Prompt {
    components: Vec<Box<dyn Component>>,
    evaluate: Option<Box<Evaluate>>,
}

static ONCE: Once = Once::new();

impl Prompt {
    pub fn new(components: Vec<Box<dyn Component>>) -> Self {
        Self {
            components,
            evaluate: None,
        }
    }

    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<Vec<String>> {
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
            self.components
                .iter()
                .map(|editor| editor.make_pane(size.0))
                .collect(),
        )?;

        loop {
            let ev = event::read()?;

            for editor in &mut self.components {
                editor.handle_event(&ev);
            }

            let finalizable = if let Some(evaluate) = &self.evaluate {
                evaluate(&ev, &self.components)?
            } else {
                true
            };

            let size = engine.size()?;
            terminal.draw(
                &mut engine,
                self.components
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
                }) => {
                    return Err(Error::Interrupted {
                        event: ev,
                    })
                }
                _ => (),
            }
        }

        let ret = self
            .components
            .iter()
            .map(|editor| editor.output())
            .collect();
        self.components.iter_mut().for_each(|editor| {
            editor.postrun();
        });
        Ok(ret)
    }
}
