use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::{future::FutureExt, stream::StreamExt, Future};

use tokio::sync::mpsc::Receiver;

use promkit::{
    crossterm::{
        cursor,
        event::{Event, EventStream},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    pane::Pane,
    terminal::Terminal,
};

mod event_buffer;
use event_buffer::EventBuffer;
pub use event_buffer::WrappedEvent;
mod resize_debounce;
use resize_debounce::ResizeDebounce;
mod display_coordinator;
use display_coordinator::DisplayCoordinator;

pub trait PaneSyncer: promkit::Finalizer {
    fn init_panes(&self, width: u16, height: u16) -> Vec<Pane>;
    fn sync(
        &mut self,
        version: usize,
        events: &[WrappedEvent],
        width: u16,
        height: u16,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub struct Prompt<T: PaneSyncer> {
    pub renderer: T,
}

impl<T: PaneSyncer> Drop for Prompt<T> {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::MoveToNextLine(1)).ok();
        execute!(io::stdout(), cursor::Show).ok();
        disable_raw_mode().ok();
    }
}

impl<T: PaneSyncer> Prompt<T> {
    pub async fn run(
        &mut self,
        event_buffer_delay_duration: Duration,
        resize_debounce_delay_duration: Duration,
        merger_delay_duration: Duration,
        mut fin_receiver: Receiver<()>,
        versioned_each_pane_receiver: Receiver<(usize, usize, Pane)>,
        versioned_loading_indicator_receiver: Receiver<(usize, usize)>,
    ) -> anyhow::Result<T::Return> {
        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        let mut size = crossterm::terminal::size()?;

        let mut event_buffer = EventBuffer::new(event_buffer_delay_duration);
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(1);
        let (event_buffer_sender, mut event_buffer_receiver) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move { event_buffer.run(event_receiver, event_buffer_sender).await });

        let resize_debouncer = ResizeDebounce::new(resize_debounce_delay_duration);
        let (resize_sender, resize_receiver) = tokio::sync::mpsc::channel(1);
        let (debounced_resize_sender, mut debounced_resize_receiver) =
            tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            resize_debouncer
                .run(resize_receiver, debounced_resize_sender)
                .await
        });

        let panes = self.renderer.init_panes(size.0, size.1);

        let mut terminal = Terminal::start_session(&panes)?;
        terminal.draw(&panes)?;

        let coordinator = DisplayCoordinator::new(terminal, merger_delay_duration, panes);
        tokio::spawn(async move {
            coordinator
                .run(
                    versioned_each_pane_receiver,
                    versioned_loading_indicator_receiver,
                )
                .await
        });

        let mut stream = EventStream::new();

        let version = Arc::new(AtomicUsize::new(1));

        loop {
            futures::select! {
                maybe_event = stream.next().fuse() => {
                    if let Some(Ok(event)) = maybe_event {
                        match event {
                            Event::Resize(width, height) => {
                                let _ = resize_sender.send((width, height)).await;
                            }
                            other => {
                                event_sender.send(other).await?;
                            }
                        }
                    }
                },
                maybe_debounced_resize = debounced_resize_receiver.recv().fuse() => {
                    if let Some((width, height)) = maybe_debounced_resize {
                        size = (width, height);
                    }
                },
                maybe_event_buffer = event_buffer_receiver.recv().fuse() => {
                    if let Some(event_buffer) = maybe_event_buffer {
                        let next = version.fetch_add(1, Ordering::SeqCst);
                        self.renderer.sync(next, &event_buffer, size.0, size.1).await?;
                    }
                },
                maybe_fin = fin_receiver.recv().fuse() => {
                    if maybe_fin.is_some() {
                        break;
                    }
                },
            }
        }

        self.renderer.finalize()
    }
}
