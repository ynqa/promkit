use std::time::Duration;

use crate::core::{Pane, grapheme::StyledGraphemes, render::SharedRenderer};

pub mod frame;
use frame::Frame;

/// Trait to define the state of the spinner.
pub trait State {
    fn is_idle(&self) -> impl Future<Output = bool> + Send;
}

/// A spinner that can be used to indicate loading or processing states.
#[derive(Clone, Debug)]
pub struct Spinner {
    /// The frames of the spinner, which are the characters that will be displayed in a rotating manner.
    pub frames: Frame,
    /// A suffix that will be displayed alongside the spinner.
    pub suffix: String,
    /// The duration between frame updates.
    pub duration: Duration,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            frames: frame::DOTS.clone(),
            suffix: String::new(),
            duration: Duration::from_millis(100),
        }
    }
}

impl Spinner {
    /// Set frames for the spinner.
    pub fn frames(mut self, frames: Frame) -> Self {
        self.frames = frames;
        self
    }

    /// Set a suffix for the spinner.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Set the duration between frame updates.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Spawn a background task that shows a spinner while the state is active.
pub async fn run<S, I>(
    spinner: &Spinner,
    state: S,
    index: I,
    renderer: SharedRenderer<I>,
) -> anyhow::Result<()>
where
    S: State,
    I: Clone + Ord + Send,
{
    let mut frame_index = 0;
    let mut interval = tokio::time::interval(spinner.duration);

    loop {
        interval.tick().await;

        if !state.is_idle().await {
            frame_index = (frame_index + 1) % spinner.frames.len();

            renderer
                .update([(
                    index.clone(),
                    Pane::new(
                        vec![StyledGraphemes::from(format!(
                            "{} {}",
                            spinner.frames[frame_index], spinner.suffix
                        ))],
                        0,
                    ),
                )])
                .render()
                .await?;
        }
    }
}
