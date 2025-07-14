use tokio::time::Duration;

use crate::core::{Pane, grapheme::StyledGraphemes, render::SharedRenderer};

#[async_trait::async_trait]
pub trait State {
    async fn is_idle(&self) -> bool;
}

/// Spawn a background task that shows a spinner while the state is active.
pub async fn run<S, I>(
    spinner: &[String],
    suffix: &str,
    duration: Duration,
    state: S,
    index: I,
    renderer: SharedRenderer<I>,
) -> anyhow::Result<()>
where
    S: State,
    I: Clone + Ord + Send,
{
    let mut frame_index = 0;
    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;

        if !state.is_idle().await {
            frame_index = (frame_index + 1) % spinner.len();

            renderer
                .update([(
                    index.clone(),
                    Pane::new(
                        vec![StyledGraphemes::from(format!(
                            "{} {}",
                            spinner[frame_index], suffix
                        ))],
                        0,
                    ),
                )])
                .render()
                .await?;
        }
    }
}
