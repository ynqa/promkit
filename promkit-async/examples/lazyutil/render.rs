use std::sync::{Arc, Mutex};

use promkit::{pane::Pane, switch::ActiveKeySwitcher, text_editor, PaneFactory};

use futures::Future;
use tokio::sync::mpsc::{self, Sender};

use promkit_async::{
    spinner::{self, Spinner},
    PaneSyncer, WrappedEvent,
};

use crate::lazyutil::keymap;

pub struct Renderer {
    keymap: ActiveKeySwitcher<keymap::Handler>,
    state: Arc<Mutex<text_editor::State>>,
    lazy_state: Arc<Mutex<text_editor::State>>,

    fin_sender: Sender<()>,
    spinner_signal_sender: Sender<spinner::Signal>,
    pane_sender: Sender<(Pane, usize)>,
}

impl Renderer {
    pub fn new(
        keymap: ActiveKeySwitcher<keymap::Handler>,
        state: text_editor::State,
        lazy_state: text_editor::State,
        fin_sender: Sender<()>,
        pane_sender: Sender<(Pane, usize)>,
    ) -> anyhow::Result<Self> {
        // Under investigation: reducing the size of the channel to a very small value
        // results in missing characters in the string rendered by lazy_renderer.
        let (spinner_signal_sender, spinner_signal_receiver) = mpsc::channel(10);
        let pane_sender_clone = pane_sender.clone();
        tokio::spawn(async move {
            Spinner::default()
                .run(1, pane_sender_clone, spinner_signal_receiver)
                .await
        });
        Ok(Self {
            keymap,
            state: Arc::new(Mutex::new(state)),
            lazy_state: Arc::new(Mutex::new(lazy_state)),
            fin_sender,
            spinner_signal_sender,
            pane_sender,
        })
    }
}

impl PaneSyncer for Renderer {
    type Return = String;

    fn init_panes(&self, width: u16) -> Vec<Pane> {
        vec![
            self.state.lock().unwrap().create_pane(width),
            self.lazy_state.lock().unwrap().create_pane(width),
        ]
    }

    fn sync(
        &mut self,
        event: &WrappedEvent,
        width: u16,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let state = Arc::clone(&self.state);
        let lazy_state = Arc::clone(&self.lazy_state);
        let fin_sender = self.fin_sender.clone();
        let pane_sender = self.pane_sender.clone();
        let spinner_signal_sender = self.spinner_signal_sender.clone();
        let event = event.clone();
        let keymap = self.keymap.clone();

        async move {
            tokio::spawn(async move {
                let mut state = state.lock().unwrap();
                keymap.get()(&event, &mut state, &fin_sender)?;
                pane_sender.try_send((state.create_pane(width), 0))?;

                let edited = state.clone();
                tokio::spawn(async move {
                    spinner_signal_sender.try_send(spinner::Signal::Activate)?;

                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    let mut lazy_state = lazy_state.lock().unwrap();
                    lazy_state.texteditor = edited.texteditor;
                    spinner_signal_sender.try_send(spinner::Signal::SuspendAndSend(
                        lazy_state.create_pane(width),
                    ))?;
                    Ok::<(), anyhow::Error>(())
                });

                Ok::<(), anyhow::Error>(())
            });

            Ok(())
        }
    }

    fn output(&self) -> anyhow::Result<Self::Return> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .texteditor
            .text_without_cursor()
            .to_string())
    }
}
