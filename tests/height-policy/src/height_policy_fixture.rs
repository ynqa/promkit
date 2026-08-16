use std::{env, io};

use futures::StreamExt;
use promkit_core::{
    crossterm::{
        cursor,
        event::{Event, EventStream, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    grapheme::StyledGraphemes,
    render::Renderer,
    CreatedGraphemes, HeightPolicy, WidgetLayout,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    First,
    Second,
    Third,
    Fourth,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        execute!(io::stdout(), cursor::Show).ok();
        disable_raw_mode().ok();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), cursor::Hide)?;
    let _terminal_guard = TerminalGuard;

    let mode = env::args()
        .nth(1)
        .unwrap_or_else(|| "ordered-content".to_owned());
    let renderer = Renderer::try_new_with_graphemes(initial_content(&mode)?, true).await?;
    let mut events = EventStream::new();

    while let Some(event) = events.next().await {
        match event? {
            Event::Key(key)
                if mode == "fair-content"
                    && key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Tab =>
            {
                renderer.update(short_fair_content()).render().await?;
            }
            Event::Key(key)
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) =>
            {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn initial_content(mode: &str) -> anyhow::Result<Vec<(Index, CreatedGraphemes)>> {
    match mode {
        "ordered-content" => Ok(ordered_content().into_iter().collect()),
        "fair-fill" => Ok(fair_fill().into_iter().collect()),
        "fair-content" => Ok(filled_fair_content().into_iter().collect()),
        _ => Err(anyhow::anyhow!("unknown height policy fixture: {mode}")),
    }
}

fn ordered_content() -> [(Index, CreatedGraphemes); 3] {
    [
        (
            Index::First,
            pane(
                "ordered-a-1\nordered-a-2\nordered-a-3\nordered-a-4\nordered-a-5",
                HeightPolicy::OrderedContent,
            ),
        ),
        (
            Index::Second,
            pane(
                "ordered-b-1\nordered-b-2\nordered-b-3\nordered-b-4\nordered-b-5",
                HeightPolicy::OrderedContent,
            ),
        ),
        (
            Index::Third,
            pane(
                "ordered-c-1\nordered-c-2\nordered-c-3\nordered-c-4\nordered-c-5",
                HeightPolicy::OrderedContent,
            ),
        ),
    ]
}

fn fair_fill() -> [(Index, CreatedGraphemes); 4] {
    [
        (Index::First, pane("header", HeightPolicy::OrderedContent)),
        (Index::Second, pane("fair-a", HeightPolicy::FairFill)),
        (Index::Third, pane("fair-b", HeightPolicy::FairFill)),
        (Index::Fourth, pane("footer", HeightPolicy::OrderedContent)),
    ]
}

fn filled_fair_content() -> [(Index, CreatedGraphemes); 2] {
    [
        (
            Index::First,
            pane(
                "upper-1\nupper-2\nupper-3\nupper-4\nupper-5\nupper-6",
                HeightPolicy::FairContent,
            ),
        ),
        (
            Index::Second,
            pane(
                "lower-1\nlower-2\nlower-3\nlower-4\nlower-5\nlower-6",
                HeightPolicy::FairContent,
            ),
        ),
    ]
}

fn short_fair_content() -> [(Index, CreatedGraphemes); 2] {
    [
        (
            Index::First,
            pane("short-1\nshort-2", HeightPolicy::FairContent),
        ),
        (
            Index::Second,
            pane(
                "long-1\nlong-2\nlong-3\nlong-4\nlong-5\nlong-6",
                HeightPolicy::FairContent,
            ),
        ),
    ]
}

fn pane(content: &str, height_policy: HeightPolicy) -> CreatedGraphemes {
    CreatedGraphemes {
        graphemes: StyledGraphemes::from(content),
        layout: WidgetLayout {
            height_policy,
            ..Default::default()
        },
        cursor: None,
    }
}
