use std::sync::{Arc, Mutex};

use promkit::{pane::Pane, switch::ActiveKeySwitcher, text_editor, PaneFactory};

use futures::Future;
use tokio::sync::mpsc::Sender;

use promkit_async::{EventBundle, PaneSyncer};

use crate::lazyutil::keymap;

pub struct Renderer {
    keymap: ActiveKeySwitcher<keymap::Handler>,
    state: Arc<Mutex<text_editor::State>>,
    lazy_state: Arc<Mutex<text_editor::State>>,

    fin_sender: Sender<()>,
    indexed_pane_sender: Sender<(usize, usize, Pane)>,
    loading_activation_sender: Sender<(usize, usize)>,
}

impl Renderer {
    pub fn new(
        keymap: ActiveKeySwitcher<keymap::Handler>,
        state: text_editor::State,
        lazy_state: text_editor::State,
        fin_sender: Sender<()>,
        indexed_pane_sender: Sender<(usize, usize, Pane)>,
        loading_activation_sender: Sender<(usize, usize)>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            keymap,
            state: Arc::new(Mutex::new(state)),
            lazy_state: Arc::new(Mutex::new(lazy_state)),
            fin_sender,
            indexed_pane_sender,
            loading_activation_sender,
        })
    }
}

impl promkit::Finalizer for Renderer {
    type Return = String;

    fn finalize(&self) -> anyhow::Result<Self::Return> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .texteditor
            .text_without_cursor()
            .to_string())
    }
}

impl PaneSyncer for Renderer {
    fn init_panes(&self, width: u16, height: u16) -> Vec<Pane> {
        vec![
            self.state.lock().unwrap().create_pane(width, height),
            self.lazy_state.lock().unwrap().create_pane(width, height),
        ]
    }

    fn sync(
        &mut self,
        version: usize,
        event_buffer: &[EventBundle],
        width: u16,
        height: u16,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let state = Arc::clone(&self.state);
        let lazy_state = Arc::clone(&self.lazy_state);
        let fin_sender = self.fin_sender.clone();
        let indexed_pane_sender = self.indexed_pane_sender.clone();
        let loading_activation_sender = self.loading_activation_sender.clone();
        let event_buffer = event_buffer.to_vec();
        let keymap = self.keymap.clone();

        async move {
            loading_activation_sender.send((version, 1)).await?;
            let mut state = state.lock().unwrap();
            keymap.get()(&event_buffer, &mut state, &fin_sender)?;
            indexed_pane_sender.try_send((version, 0, state.create_pane(width, height)))?;

            let edited = state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                let mut lazy_state = lazy_state.lock().unwrap();
                lazy_state.texteditor = edited.texteditor;
                indexed_pane_sender.try_send((
                    version,
                    1,
                    lazy_state.create_pane(width, height),
                ))?;
                Ok::<(), anyhow::Error>(())
            });

            Ok(())
        }
    }
}
