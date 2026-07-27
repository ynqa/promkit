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

/// Singleton event stream shared by all prompts.
pub static EVENT_STREAM: LazyLock<Mutex<EventStream>> =
    LazyLock::new(|| Mutex::new(EventStream::new()));

/// Controls whether a prompt continues processing events or exits.
#[derive(Eq, PartialEq)]
pub enum Signal {
    Continue,
    Quit,
}

/// Lifecycle for an interactive prompt.
#[async_trait::async_trait]
pub trait Prompt {
    async fn initialize(&mut self) -> anyhow::Result<()>;

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal>;

    type Return;

    fn finalize(&mut self) -> anyhow::Result<Self::Return>;

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
