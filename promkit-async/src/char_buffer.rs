use futures::future::{Future, FutureExt};
use futures_timer::Delay;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct CharBuffer {
    buffer: Vec<char>,
    delay_duration: Duration,
}

impl CharBuffer {
    pub fn new() -> Self {
        CharBuffer {
            buffer: Vec::new(),
            delay_duration: Duration::from_millis(10),
        }
    }

    pub fn run(
        &mut self,
        mut char_receiver: Receiver<char>,
        buffer_sender: Sender<Vec<char>>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        let mut buffer = self.buffer.clone();
        let delay_duration = self.delay_duration;

        async move {
            loop {
                let delay = Delay::new(delay_duration).fuse();
                futures::pin_mut!(delay);

                futures::select! {
                    char_opt = char_receiver.recv().fuse() => {
                        if let Some(c) = char_opt {
                            buffer.push(c);
                        } else {
                            break;
                        }
                    },
                    _ = delay => {
                        if !buffer.is_empty() {
                            buffer_sender.send(buffer.clone()).await?;
                            buffer.clear();
                        }
                    },
                }
            }
            Ok(())
        }
    }
}
