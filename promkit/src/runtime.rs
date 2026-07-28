//! A small lifecycle runtime for sequential, interactive prompts.
//!
//! This module owns terminal setup, teardown, and event delivery. Applications
//! remain responsible for composing widgets, mapping events to state changes,
//! rendering updates, focus management, validation, and quit policy.

use std::{io, sync::LazyLock};

use futures::StreamExt;
use scopeguard::defer;
use tokio::sync::Mutex;

use crate::core::crossterm::{
    cursor,
    event::{self, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

/// The singleton terminal event stream used by [`Prompt::run`].
///
/// A single stream is shared because repeatedly constructing an [`EventStream`]
/// can cause crossterm cursor-position reads to time out. See
/// [crossterm issue #963](https://github.com/crossterm-rs/crossterm/issues/963#issuecomment-2571259264).
///
/// Access is serialized by a [`Mutex`]. Consequently, [`Prompt::run`] is
/// intended for one active prompt at a time, not for concurrently running
/// prompts. Applications that need to combine terminal events with background
/// work or application events should own their event loop instead.
pub static EVENT_STREAM: LazyLock<Mutex<EventStream>> =
    LazyLock::new(|| Mutex::new(EventStream::new()));

/// Indicates whether [`Prompt::run`] should continue reading terminal events.
#[derive(Eq, PartialEq)]
pub enum Signal {
    /// Continue reading and evaluating terminal events.
    Continue,
    /// Stop the event loop and call [`Prompt::finalize`].
    Quit,
}

/// Defines the lifecycle of a sequential, interactive prompt.
///
/// `Prompt` provides lifecycle hooks around the event loop implemented by
/// [`Prompt::run`]:
///
/// 1. [`Prompt::initialize`] prepares and normally renders the initial state.
/// 2. [`Prompt::evaluate`] handles each non-resize terminal event in order.
/// 3. [`Prompt::finalize`] produces the prompt's return value after the loop
///    ends.
///
/// Implementors own their widget state, renderer, key bindings, focus
/// transitions, validation, and quit conditions. This trait does not prescribe
/// how an event changes a widget and is not intended to replace an
/// application-specific event loop.
#[async_trait::async_trait]
pub trait Prompt {
    /// Prepares the prompt before terminal events are evaluated.
    ///
    /// [`Prompt::run`] calls this after enabling raw mode, hiding the cursor,
    /// and enabling mouse capture. Implementations commonly initialize their
    /// renderer and draw the initial prompt here.
    ///
    /// If this method returns an error, the error is propagated and
    /// [`Prompt::finalize`] is not called. Terminal modes configured by
    /// [`Prompt::run`] are still restored.
    async fn initialize(&mut self) -> anyhow::Result<()>;

    /// Applies one terminal event to the application-owned prompt state.
    ///
    /// Events are evaluated sequentially. Resize events are skipped by
    /// [`Prompt::run`] and are not passed to this method. Implementations are
    /// responsible for mapping the event to state changes and rendering any
    /// resulting UI update.
    ///
    /// Return [`Signal::Continue`] to wait for another event or
    /// [`Signal::Quit`] to leave the loop and call [`Prompt::finalize`]. If this
    /// method returns an error, the error is propagated and
    /// [`Prompt::finalize`] is not called.
    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal>;

    /// The value produced by [`Prompt::finalize`] and returned from
    /// [`Prompt::run`].
    type Return;

    /// Produces the final value after the event loop ends.
    ///
    /// [`Prompt::run`] calls this after [`Signal::Quit`], the event stream ends,
    /// or reading the event stream fails. It is not called when
    /// [`Prompt::initialize`] or [`Prompt::evaluate`] returns an error.
    fn finalize(&mut self) -> anyhow::Result<Self::Return>;

    /// Runs the prompt lifecycle and returns its final value.
    ///
    /// This method:
    ///
    /// 1. enables raw mode, hides the cursor, and enables mouse capture;
    /// 2. calls [`Prompt::initialize`];
    /// 3. reads events from [`EVENT_STREAM`] and passes non-resize events to
    ///    [`Prompt::evaluate`] until it returns [`Signal::Quit`];
    /// 4. calls [`Prompt::finalize`]; and
    /// 5. restores cursor visibility, disables mouse capture, and disables raw
    ///    mode.
    ///
    /// Terminal restoration is attempted when the method exits, including
    /// error paths. Restoration errors are ignored so that an earlier prompt
    /// error is preserved.
    ///
    /// # Runtime scope
    ///
    /// The runtime writes terminal commands to standard output and processes
    /// one prompt sequentially. It does not enter or leave the alternate
    /// screen, redraw immediately on resize, multiplex application events, or
    /// coordinate concurrent prompts. Applications that need those policies
    /// should own their event loop and use `promkit-core` and
    /// `promkit-widgets` directly.
    ///
    /// # Errors
    ///
    /// Returns errors from terminal setup, [`Prompt::initialize`],
    /// [`Prompt::evaluate`], or [`Prompt::finalize`]. An error yielded by the
    /// terminal event stream ends the loop and is not propagated; in that case,
    /// [`Prompt::finalize`] determines the result.
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
        execute!(io::stdout(), cursor::Hide, event::EnableMouseCapture)?;

        self.initialize().await?;

        while let Some(event) = EVENT_STREAM.lock().await.next().await {
            match event {
                Ok(event) => {
                    if event.is_resize() {
                        continue;
                    }
                    if self.evaluate(&event).await? == Signal::Quit {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        self.finalize()
    }
}
