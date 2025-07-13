#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use promkit_widgets as widgets;
pub use promkit_widgets::core;

pub mod preset;
pub mod suggest;
pub mod validate;

use std::{io, sync::LazyLock};

use futures::StreamExt;
use scopeguard::defer;
use tokio::sync::Mutex;

use core::{
    crossterm::{
        cursor,
        event::{self, Event, EventStream},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    render::SharedRenderer,
};

/// Singleton for EventStream. If a new EventStream is created for each Prompt::run,
/// it causes the error "The cursor position could not be read within a normal duration".
/// See https://github.com/crossterm-rs/crossterm/issues/963#issuecomment-2571259264 for more details.
static EVENT_STREAM: LazyLock<Mutex<EventStream>> =
    LazyLock::new(|| Mutex::new(EventStream::new()));

/// Represents the signal to control the flow of a prompt.
///
/// This enum is used to indicate whether a prompt should continue running
/// or quit based on user input or other conditions.
#[derive(Eq, PartialEq)]
pub enum Signal {
    /// Indicates that the prompt should continue to run and handle further events.
    Continue,
    /// Indicates that the prompt should quit and terminate its execution.
    Quit,
}

/// A trait for rendering components within a prompt.
///
/// This trait defines the essential functions required for rendering custom UI components
/// in a prompt. Implementors of this trait can define how panes are created, how events
/// are evaluated, and how the final result is produced.
#[async_trait::async_trait]
pub trait Prompt {
    /// The type of index used to identify different components in the prompt.
    type Index: Ord + Send + Sync + 'static;

    /// Returns a shared renderer for the prompt.
    fn renderer(&self) -> SharedRenderer<Self::Index>;

    /// Initializes the handler, preparing it for use.
    /// This method is called before the prompt starts running.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the initialization.
    /// If successful, the renderer is ready to handle events and render the prompt.
    async fn initialize(&mut self) -> anyhow::Result<()>;

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
    /// Returns a `Result` containing a `Signal`. `Signal::Continue` indicates
    /// that the prompt should continue running, while `Signal::Quit` indicates that
    /// the prompt should terminate its execution.
    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal>;

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

    /// Runs the prompt, handling events and producing a result.
    ///
    /// This method initializes the terminal, and enters a loop
    /// to handle events until a quit signal is received.
    /// After exiting the loop, it produces and returns the result.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the produced result or an error.
    async fn run(&mut self) -> anyhow::Result<Self::Return> {
        defer! {
            execute!(
                io::stdout(),
                cursor::Show,
                event::DisableMouseCapture,
            )
            .ok();
            disable_raw_mode().ok();
        };

        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        self.initialize().await?;

        while let Some(event) = EVENT_STREAM.lock().await.next().await {
            match event {
                Ok(event) => {
                    // Evaluate the event using the engine
                    if self.evaluate(&event).await? == Signal::Quit {
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        self.finalize()
    }
}
