use std::time::Duration;

use futures::future::Future;
use futures_timer::Delay;

use tokio::sync::mpsc::{Receiver, Sender};
pub struct ResizeDebounce {
    delay_duration: Duration,
}

impl ResizeDebounce {
    pub fn new(delay_duration: Duration) -> Self {
        ResizeDebounce { delay_duration }
    }

    pub fn run(
        &self,
        mut resize_receiver: Receiver<(u16, u16)>,
        resize_sender: Sender<(u16, u16)>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let delay_duration = self.delay_duration;

        async move {
            let mut last_event: Option<(u16, u16)> = None;
            loop {
                let delay = Delay::new(delay_duration);
                futures::pin_mut!(delay);

                tokio::select! {
                    resize_opt = resize_receiver.recv() => {
                        if let Some(event) = resize_opt {
                            last_event = Some(event);
                        } else {
                            break;
                        }
                    },
                    _ = delay => {
                        if let Some(event) = last_event.take() {
                            resize_sender.send(event).await?;
                        }
                    },
                }
            }
            Ok(())
        }
    }
}
