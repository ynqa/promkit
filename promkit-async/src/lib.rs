use std::{
    io,
    sync::{Arc, Mutex},
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
pub use event_buffer::{EventBuffer, WrappedEvent};
mod resize_debounce;
use resize_debounce::ResizeDebounce;
mod merge;
use merge::PaneMerger;
pub mod spinner;

pub trait PaneSyncer: promkit::Finalizer {
    fn init_panes(&self, width: u16, height: u16) -> Vec<Pane>;
    fn sync(
        &mut self,
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
        mut fin_receiver: Receiver<()>,
        mut pane_receiver: Receiver<(Pane, usize)>,
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

        let mut merger = PaneMerger::new(self.renderer.init_panes(size.0, size.1));
        let mut terminal = Terminal::start_session(&merger.panes)?;
        terminal.draw(&merger.panes)?;
        let shared_terminal = Arc::new(Mutex::new(terminal));

        let mut stream = EventStream::new();

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
                        self.renderer.sync(&event_buffer, size.0, size.1).await?;
                    }
                },
                maybe_fin = fin_receiver.recv().fuse() => {
                    if maybe_fin.is_some() {
                        break;
                    }
                },
                maybe_pane = pane_receiver.recv().fuse() => {
                    match maybe_pane {
                        Some((pane, index)) => {
                            let mut terminal = shared_terminal.lock().unwrap();
                            let panes = merger.merge(index, pane);
                            terminal.draw(panes)?;
                        },
                        None => break,
                    }
                },
            }
        }

        self.renderer.finalize()
    }
}
