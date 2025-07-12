#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use promkit_widgets;
pub use promkit_widgets::core::crossterm;

pub mod preset;
pub mod snapshot;
pub mod suggest;
pub mod switch;
pub mod validate;

use std::io;

use promkit_widgets::core::{
    crossterm::{
        cursor,
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    terminal::Terminal,
    Pane,
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
}

impl<T: Renderer> Drop for Prompt<T> {
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
        execute!(io::stdout(), cursor::Hide)?;

        let size = crossterm::terminal::size()?;
        let panes = self.renderer.create_panes(size.0, size.1);
        let mut terminal = Terminal {
            position: crossterm::cursor::position()?,
        };
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
