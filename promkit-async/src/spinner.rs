use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::future::{Future, FutureExt};
use futures_timer::Delay;
use tokio::sync::mpsc::{Receiver, Sender};

use promkit::{
    crossterm::terminal,
    grapheme::{matrixify, StyledGraphemes},
    pane::Pane,
    style::StyleBuilder,
};

pub struct Spinner {
    frames: Vec<String>,
    active: Arc<AtomicBool>,
}

pub enum Signal {
    Activate,
    SuspendAndSend(Pane),
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Spinner {
    pub fn run(
        &mut self,
        index: usize,
        pane_sender: Sender<(Pane, usize)>,
        mut signal_receiver: Receiver<Signal>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let spinner_duration = Duration::from_millis(10);
        let frames = self.frames.clone();
        let active = self.active.clone();
        let pane_sender = pane_sender.clone();

        async move {
            loop {
                let delay = Delay::new(spinner_duration).fuse();
                futures::pin_mut!(delay);

                futures::select! {
                    signal = signal_receiver.recv().fuse() => match signal {
                        Some(Signal::Activate) => {
                            active.store(true, Ordering::SeqCst);
                        },
                        Some(Signal::SuspendAndSend(pane)) => {
                            active.store(false, Ordering::SeqCst);
                            let _ = pane_sender.try_send((pane, index));
                        },
                        None => break,
                    },
                    _ = delay => {
                        if active.load(Ordering::SeqCst) {
                            for frame in &frames {
                                if !active.load(Ordering::SeqCst) {
                                    break;
                                }
                                let (width, height) = terminal::size()?;
                                let _ = pane_sender.try_send((Pane::new(
                                    matrixify(
                                        width as usize,
                                        height as usize,
                                        0,
                                        &StyledGraphemes::from_str(
                                            frame,
                                            StyleBuilder::new().build(),
                                        ),
                                    ),
                                    0,
                                ), index));
                                Delay::new(spinner_duration).await;
                            }
                        }
                    },
                }
            }
            Ok(())
        }
    }
}
