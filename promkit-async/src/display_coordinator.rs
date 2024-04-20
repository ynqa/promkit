use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use crossterm::terminal;
use futures::{Future, FutureExt};
use futures_timer::Delay;
use tokio::sync::mpsc::Receiver;

use promkit::{
    grapheme::{matrixify, StyledGraphemes},
    pane::Pane,
    style::StyleBuilder,
    terminal::Terminal,
};

pub struct DisplayCoordinator {
    shared_terminal: Arc<Mutex<Terminal>>,
    version: Arc<AtomicUsize>,
    panes: Arc<Mutex<Vec<Pane>>>,
    delay_duration: Duration,
    frames: Vec<String>,
    actives: Vec<Arc<AtomicBool>>,
    frame_indexes: Vec<Arc<AtomicUsize>>,
    loading_indicator_enabled: Vec<usize>,
}

impl DisplayCoordinator {
    pub fn new(
        terminal: Terminal,
        delay_duration: Duration,
        panes: Vec<Pane>,
        loading_indicator_enabled: Vec<usize>,
    ) -> Self {
        let actives = {
            let mut v = Vec::with_capacity(panes.len());
            (0..panes.len()).for_each(|_| v.push(Arc::new(AtomicBool::new(false))));
            v
        };
        let frame_indexes = {
            let mut v = Vec::with_capacity(panes.len());
            (0..panes.len()).for_each(|_| v.push(Arc::new(AtomicUsize::new(0))));
            v
        };
        Self {
            shared_terminal: Arc::new(Mutex::new(terminal)),
            version: Arc::new(AtomicUsize::new(0)),
            panes: Arc::new(Mutex::new(panes)),
            delay_duration,
            frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            actives,
            frame_indexes,
            loading_indicator_enabled,
        }
    }

    pub fn run(
        &self,
        mut version_change_receiver: Receiver<usize>,
        mut pane_receiver: Receiver<(usize, usize, Pane)>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let global = self.version.clone();
        let shared_panes = Arc::clone(&self.panes);
        let delay_duration = self.delay_duration;
        let mut actives = self.actives.clone();
        let frames = self.frames.clone();
        let frame_indexes = self.frame_indexes.clone();
        let shared_terminal = Arc::clone(&self.shared_terminal);
        let loading_indicator_enabled = self.loading_indicator_enabled.clone();

        async move {
            loop {
                let delay = Delay::new(delay_duration).fuse();
                futures::pin_mut!(delay);

                futures::select! {
                    maybe_version = version_change_receiver.recv().fuse() => {
                        match maybe_version {
                            Some(version) => {
                                if version > global.load(Ordering::SeqCst) {
                                    global.store(version, Ordering::SeqCst);
                                    actives.iter_mut().for_each(|active| active.store(true, Ordering::SeqCst));
                                }
                            }
                            None => break,
                        }
                    },
                    maybe_triplet = pane_receiver.recv().fuse() => {
                        match maybe_triplet {
                            Some((version, index, pane)) => {
                                if version >= global.load(Ordering::SeqCst) {
                                    let mut panes = shared_panes.lock().unwrap();
                                    actives[index].store(false, Ordering::SeqCst);
                                    panes[index] = pane;
                                    shared_terminal.lock().unwrap().draw(&panes)?;
                                }
                            }
                            None => break,
                        }
                    },
                    _ = delay => {
                        let tasks: Vec<_> = actives
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| loading_indicator_enabled.contains(index))
                            .map(|(index, active)| {
                                let frames = frames.clone();
                                let shared_panes = Arc::clone(&shared_panes);
                                let frame_indexes = frame_indexes.clone();
                                let shared_terminal = Arc::clone(&shared_terminal);
                                async move {
                                    if active.load(Ordering::SeqCst) {
                                        let frame_index = frame_indexes[index].load(Ordering::SeqCst);
                                        let frame = &frames[frame_index % frames.len()];
                                        let (width, height) = terminal::size()?;
                                        let (matrix, _) = matrixify(
                                            width as usize,
                                            height as usize,
                                            0,
                                            &StyledGraphemes::from_str(
                                                frame,
                                                StyleBuilder::new().build(),
                                            ),
                                        );
                                        let mut panes = shared_panes.lock().unwrap();
                                        panes[index] = Pane::new(matrix, 0);
                                        shared_terminal.lock().unwrap().draw(&panes)?;
                                        frame_indexes[index].store((frame_index + 1) % frames.len(), Ordering::SeqCst);
                                    }
                                    Ok::<(), anyhow::Error>(())
                                }
                            })
                        .collect();
                        futures::future::join_all(tasks).await;
                    },
                }
            }
            Ok(())
        }
    }
}
