use std::{collections::HashSet, sync::Arc, time::Duration};

use promkit::{
    async_trait,
    core::crossterm::{self, style::Color},
    widgets::{
        core::{
            crossterm::{
                event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
                style::ContentStyle,
            },
            render::{Renderer, SharedRenderer},
            PaneFactory,
        },
        spinner, text::{self, Text}, text_editor,
    },
    Prompt, Signal,
};
use tokio::sync::RwLock;

/// BYOP (Bring Your Own Preset) example for promkit.

/// Represents the indices of various components in BYOP.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Readline = 0,
    Spinner = 1,
    Result = 2,
}

/// Represents the state of the task.
#[derive(Clone, PartialEq, Eq)]
enum TaskState {
    Idle,
    Running,
}

/// Shared state for the task, allowing concurrent access.
#[derive(Clone)]
struct SharedTaskState(Arc<RwLock<TaskState>>);

impl spinner::State for SharedTaskState {
    async fn is_idle(&self) -> bool {
        *self.0.read().await == TaskState::Idle
    }
}

/// Bring Your Own Prompt
struct BYOP {
    renderer: SharedRenderer<Index>,
    task_state: SharedTaskState,
    readline: text_editor::State,
    result: text::State,
}

#[async_trait::async_trait]
impl Prompt for BYOP {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let ret = self.evaluate_internal(event).await;
        let size = crossterm::terminal::size()?;
        self.render(size.0, size.1).await?;
        ret
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        let ret = self.readline.texteditor.text_without_cursor().to_string();

        // Reset the text editor state for the next prompt.
        self.readline.texteditor.erase_all();

        Ok(ret)
    }
}

impl BYOP {
    async fn try_default() -> anyhow::Result<Self> {
        let size = crossterm::terminal::size()?;

        let readline = text_editor::State {
            prefix: String::from("❯❯ "),
            prefix_style: ContentStyle {
                foreground_color: Some(Color::DarkGreen),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            word_break_chars: HashSet::from([' ']),
            ..Default::default()
        };

        let result = text::State::default();

        Ok(Self {
            renderer: SharedRenderer::new(
                Renderer::try_new_with_panes(
                    [
                        (Index::Readline, readline.create_pane(size.0, size.1)),
                        (Index::Result, result.create_pane(size.0, size.1)),
                    ],
                    true,
                )
                .await?,
            ),
            task_state: SharedTaskState(Arc::new(RwLock::new(TaskState::Idle))),
            readline,
            result,
        })
    }

    async fn heavy_task(&mut self) -> anyhow::Result<()> {
        {
            let mut task_state = self.task_state.0.write().await;
            *task_state = TaskState::Running;
        };

        // NOTE: Simulating a heavy task with a sleep.
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Update the result state after the task is done.
        self.result.text = Text::from(format!(
            "result: {}",
            self.readline.texteditor.text_without_cursor()
        ));

        {
            let mut task_state = self.task_state.0.write().await;
            *task_state = TaskState::Idle;
        };
        Ok(())
    }

    async fn evaluate_internal(&mut self, event: &Event) -> anyhow::Result<Signal> {
        match event {
            // Render for refreshing prompt on resize.
            Event::Resize(width, height) => {
                self.render(*width, *height).await?;
            }

            // Run the heavy task.
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.heavy_task().await?;
                // For representing the end of the prompt,
                // reset the style of the cursor to default.
                self.readline.active_char_style = ContentStyle::default();
                return Ok(Signal::Quit);
            }

            // Quit
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Err(anyhow::anyhow!("ctrl+c")),

            // Move cursor.
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.readline.texteditor.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.readline.texteditor.forward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.readline.texteditor.move_to_head(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.readline.texteditor.move_to_tail(),

            // Move cursor to the nearest character.
            Event::Key(KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .readline
                .texteditor
                .move_to_previous_nearest(&self.readline.word_break_chars),

            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .readline
                .texteditor
                .move_to_next_nearest(&self.readline.word_break_chars),

            // Erase char(s).
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.readline.texteditor.erase(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.readline.texteditor.erase_all(),

            // Erase to the nearest character.
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .readline
                .texteditor
                .erase_to_previous_nearest(&self.readline.word_break_chars),

            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::ALT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self
                .readline
                .texteditor
                .erase_to_next_nearest(&self.readline.word_break_chars),

            // Input char.
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.readline.texteditor.insert(*ch),

            _ => (),
        }
        Ok(Signal::Continue)
    }

    async fn render(&mut self, width: u16, height: u16) -> anyhow::Result<()> {
        self.renderer
            .update([
                (Index::Readline, self.readline.create_pane(width, height)),
                (Index::Result, self.result.create_pane(width, height)),
            ])
            .render()
            .await
    }

    async fn spawn(&mut self) -> anyhow::Result<()> {
        let task_state = self.task_state.clone();
        let renderer = self.renderer.clone();
        let spinner_task = tokio::spawn(async move {
            spinner::run(
                spinner::frame::DOTS.clone(),
                "Executing...",
                Duration::from_millis(100),
                task_state.clone(),
                Index::Spinner,
                renderer.clone(),
            )
            .await
        });

        let ret = self.run().await;
        spinner_task.abort();
        ret?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        BYOP::try_default().await?.spawn().await?
    }
}
