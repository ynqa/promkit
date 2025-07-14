use std::{collections::HashSet, sync::Arc, time::Duration};

use anyhow::Result;
use promkit::{
    async_trait,
    core::{
        crossterm::{self, style::Color},
        grapheme::StyledGraphemes,
        pane::EMPTY_PANE,
        Pane,
    },
    widgets::{
        core::{
            crossterm::{
                event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
                style::ContentStyle,
            },
            render::{Renderer, SharedRenderer},
            PaneFactory,
        },
        spinner, text_editor,
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
    task_handle: Option<tokio::task::JoinHandle<Result<String>>>,
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

        // Clean up any running task
        if let Some(handle) = &self.task_handle {
            handle.abort();
        }

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

        Ok(Self {
            renderer: SharedRenderer::new(
                Renderer::try_new_with_panes(
                    [(Index::Readline, readline.create_pane(size.0, size.1))],
                    true,
                )
                .await?,
            ),
            task_state: SharedTaskState(Arc::new(RwLock::new(TaskState::Idle))),
            readline,
            task_handle: None,
        })
    }

    async fn start_heavy_task(&mut self) -> anyhow::Result<()> {
        // Check if task is already running
        if *self.task_state.0.read().await == TaskState::Running {
            return Ok(());
        }

        // Clear previous result and show spinner
        self.renderer
            .update([(Index::Result, EMPTY_PANE.clone())])
            .render()
            .await?;

        let input_text = self.readline.texteditor.text_without_cursor().to_string();
        let task_state = self.task_state.clone();
        let renderer = self.renderer.clone();

        {
            let mut state = task_state.0.write().await;
            *state = TaskState::Running;
        }

        let handle = tokio::spawn(async move {
            // NOTE: Simulating a heavy task with a sleep.
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Set task state to idle
            {
                let mut state = task_state.0.write().await;
                *state = TaskState::Idle;
            }

            // Trigger a render to show the result
            renderer
                .update([
                    (Index::Spinner, EMPTY_PANE.clone()),
                    (
                        Index::Result,
                        Pane::new(
                            vec![StyledGraphemes::from(format!("result: {}", input_text,))],
                            0,
                        ),
                    ),
                ])
                .render()
                .await?;

            Ok(input_text)
        });

        self.task_handle = Some(handle);
        Ok(())
    }

    async fn evaluate_internal(&mut self, event: &Event) -> anyhow::Result<Signal> {
        match event {
            // Render for refreshing prompt on resize.
            Event::Resize(width, height) => {
                self.render(*width, *height).await?;
            }

            // Handle Enter key based on task state
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                let current_state = self.task_state.0.read().await.clone();
                match current_state {
                    TaskState::Idle => {
                        // Start the heavy task in background
                        self.start_heavy_task().await?;
                    }
                    TaskState::Running => {
                        self.renderer
                            .update([(
                                Index::Result,
                                Pane::new(
                                    vec![StyledGraphemes::from("Task is currently running...")],
                                    0,
                                ),
                            )])
                            .render()
                            .await?;
                    }
                }
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
            .update([(Index::Readline, self.readline.create_pane(width, height))])
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
