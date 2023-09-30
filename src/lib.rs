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

use event_handler::EventHandler;
mod engine;
mod event_handler;
mod grapheme;
mod pane;
mod terminal;
mod text;

use std::io;
use std::sync::Once;

use anyhow::{bail, Result};
use scopeguard::defer;

use crate::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    engine::Engine,
    grapheme::Graphemes,
    pane::Pane,
    text::TextBuffer,
};

/// A core data structure to manage the hooks and state.
pub struct Prompt {
    engine: Engine<io::Stdout>,
    event_handler: EventHandler,
}

static ONCE: Once = Once::new();

impl Prompt {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(io::stdout()),
            event_handler: EventHandler {},
        }
    }

    /// Loop the steps that receive an event and trigger the handler.
    pub fn run(&mut self) -> Result<String> {
        ONCE.call_once(|| {
            self.engine.clear().ok();
        });

        enable_raw_mode()?;
        defer! {{
            disable_raw_mode().ok();
        }};

        let mut textbuffer = TextBuffer::new();

        loop {
            let ev = event::read()?;
            match &ev {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => bail!("ctrl+c interrupted"),
                _ => (),
            }
            match self.event_handler.handle_event(&ev, &mut textbuffer) {
                Some(diff) => {
                    self.engine.clear()?;
                    let rendered =
                        Pane {}.render(self.engine.size()?, &diff[1], &Graphemes::from("❯❯ "))?;
                    for row in rendered {
                        self.engine.write(&row)?;
                    }
                }
                None => break,
            }
        }

        self.engine.move_to_next_line(false)?;
        Ok(textbuffer.to_string())
    }
}
