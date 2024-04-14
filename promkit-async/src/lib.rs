use std::{
    io,
    sync::{Arc, Mutex},
};

use futures::{future::FutureExt, stream::StreamExt, Future};

use tokio::sync::mpsc::Receiver;

use promkit::{
    crossterm::{
        cursor,
        event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    pane::Pane,
    terminal::Terminal,
};

mod char_buffer;
use char_buffer::CharBuffer;
mod resize_debounce;
use resize_debounce::ResizeDebounce;
mod merge;
use merge::PaneMerger;
pub mod spinner;

#[derive(Clone)]
pub enum WrappedEvent {
    KeyBuffer(Vec<char>),
    Other(Event),
}

pub trait PaneSyncer {
    type Return;
    fn init_panes(&self, width: u16) -> Vec<Pane>;
    fn sync(
        &mut self,
        event: &WrappedEvent,
        width: u16,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn output(&self) -> anyhow::Result<Self::Return>;
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
        mut fin_receiver: Receiver<()>,
        mut pane_receiver: Receiver<(Pane, usize)>,
    ) -> anyhow::Result<T::Return> {
        enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;

        let mut size = crossterm::terminal::size()?;

        let mut char_buffer = CharBuffer::new();
        let (char_sender, char_receiver) = tokio::sync::mpsc::channel(1);
        let (char_buffer_sender, mut char_buffer_receiver) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move { char_buffer.run(char_receiver, char_buffer_sender).await });

        let resize_debouncer = ResizeDebounce::new();
        let (resize_sender, resize_receiver) = tokio::sync::mpsc::channel(1);
        let (debounced_resize_sender, mut debounced_resize_receiver) =
            tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            resize_debouncer
                .run(resize_receiver, debounced_resize_sender)
                .await
        });

        let mut merger = PaneMerger::new(self.renderer.init_panes(size.0));
        // let terminal = Terminal::init_draw(merger.panes)?;
        let mut terminal = Terminal::start_session(&merger.panes)?;
        terminal.draw(&merger.panes)?;
        let shared_terminal = Arc::new(Mutex::new(terminal));

        let mut stream = EventStream::new();

        loop {
            futures::select! {
                maybe_event = stream.next().fuse() => {
                    if let Some(Ok(event)) = maybe_event {
                        match event {
                            Event::Key(KeyEvent {
                                code: KeyCode::Char(c),
                                modifiers: KeyModifiers::NONE,
                                kind: KeyEventKind::Press,
                                state: KeyEventState::NONE,
                            }) => {
                                let _ = char_sender.send(c).await;
                            },
                            Event::Resize(width, height) => {
                                let _ = resize_sender.send((width, height)).await;
                            }
                            other => {
                                self.renderer.sync(&WrappedEvent::Other(other), size.0).await?;
                            }
                        }
                    }
                },
                maybe_debounced_resize = debounced_resize_receiver.recv().fuse() => {
                    if let Some((width, height)) = maybe_debounced_resize {
                        size = (width, height);
                    }
                },
                maybe_char_buffer = char_buffer_receiver.recv().fuse() => {
                    if let Some(char_buffer) = maybe_char_buffer {
                        let size = crossterm::terminal::size()?;
                        self.renderer.sync(&WrappedEvent::KeyBuffer(char_buffer), size.0).await?;
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
                            let local_terminal = Arc::clone(&shared_terminal);
                            let mut terminal = local_terminal.lock().unwrap();
                            let panes = merger.merge(index, pane);
                            terminal.draw(panes)?;
                        },
                        None => break,
                    }
                },
            }
        }

        self.renderer.output()
    }
}
