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
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

/// BYOP (Bring Your Own Preset) example for promkit.

/// Represents the indices of various components in BYOP.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Readline = 0,
    Spinner = 1,
    Result = 2,
}

/// Task events for monitor
#[derive(Debug)]
enum TaskEvent {
    TaskStarted { handle: JoinHandle<Result<String>> },
    TaskCompleted { result: Result<String> },
}

/// Task monitor daemon for managing background tasks
struct TaskMonitor {
    event_sender: mpsc::UnboundedSender<TaskEvent>,
    _monitor_handle: JoinHandle<()>,
}

impl TaskMonitor {
    fn new(renderer: SharedRenderer<Index>, shared_task_handle: SharedTaskHandle) -> Self {
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

        // Event handling daemon
        let monitor_handle = tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    TaskEvent::TaskStarted { handle } => {
                        // Store the handle for spinner state management
                        {
                            let mut handle_guard = shared_task_handle.0.write().await;
                            *handle_guard = Some(handle);
                        }
                    }
                    TaskEvent::TaskCompleted { result } => {
                        // Clear the task handle to indicate completion
                        {
                            let mut handle_guard = shared_task_handle.0.write().await;
                            *handle_guard = None;
                        }

                        // Update UI based on result
                        match result {
                            Ok(input_text) => {
                                let _ = renderer
                                    .update([
                                        (Index::Spinner, EMPTY_PANE.clone()),
                                        (
                                            Index::Result,
                                            Pane::new(
                                                vec![StyledGraphemes::from(format!(
                                                    "result: {}",
                                                    input_text
                                                ))],
                                                0,
                                            ),
                                        ),
                                    ])
                                    .render()
                                    .await;
                            }
                            Err(_) => {
                                let _ = renderer
                                    .update([
                                        (Index::Spinner, EMPTY_PANE.clone()),
                                        (
                                            Index::Result,
                                            Pane::new(
                                                vec![StyledGraphemes::from("Task failed")],
                                                0,
                                            ),
                                        ),
                                    ])
                                    .render()
                                    .await;
                            }
                        }
                    }
                }
            }
        });

        Self {
            event_sender,
            _monitor_handle: monitor_handle,
        }
    }
}

/// Shared task handle for managing task state.
#[derive(Clone)]
struct SharedTaskHandle(Arc<RwLock<Option<JoinHandle<Result<String>>>>>);

impl spinner::State for SharedTaskHandle {
    async fn is_idle(&self) -> bool {
        self.0.read().await.is_none()
    }
}

/// Bring Your Own Prompt
struct BYOP {
    renderer: SharedRenderer<Index>,
    shared_task_handle: SharedTaskHandle,
    task_monitor: TaskMonitor,
    readline: text_editor::State,
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
        if let Some(handle) = self.shared_task_handle.0.blocking_read().as_ref() {
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

        let renderer = SharedRenderer::new(
            Renderer::try_new_with_panes(
                [(Index::Readline, readline.create_pane(size.0, size.1))],
                true,
            )
            .await?,
        );

        let shared_task_handle = SharedTaskHandle(Arc::new(RwLock::new(None)));
        let task_monitor = TaskMonitor::new(renderer.clone(), shared_task_handle.clone());

        Ok(Self {
            renderer,
            shared_task_handle,
            task_monitor,
            readline,
        })
    }

    async fn start_heavy_task(&mut self) -> anyhow::Result<()> {
        // Check if task is already running
        if self.shared_task_handle.0.read().await.is_some() {
            return Ok(());
        }

        // Clear previous result and show spinner
        self.renderer
            .update([(Index::Result, EMPTY_PANE.clone())])
            .render()
            .await?;

        let input_text = self.readline.texteditor.text_without_cursor().to_string();
        let task_monitor = self.task_monitor.event_sender.clone();

        let handle = tokio::spawn(async move {
            // NOTE: Simulating a heavy task with a sleep.
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Notify completion through the task monitor
            let result = Ok(input_text.clone());
            let _ = task_monitor.send(TaskEvent::TaskCompleted { result });

            Ok(input_text)
        });

        // Send task started event to monitor for handle management
        let _ = self
            .task_monitor
            .event_sender
            .send(TaskEvent::TaskStarted { handle });

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
                let is_running = self.shared_task_handle.0.read().await.is_some();
                if !is_running {
                    // Start the heavy task in background
                    self.start_heavy_task().await?;
                } else {
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
        let shared_task_handle = self.shared_task_handle.clone();
        let renderer = self.renderer.clone();
        let spinner_task = tokio::spawn(async move {
            spinner::run(
                spinner::frame::DOTS.clone(),
                "Executing...",
                Duration::from_millis(100),
                shared_task_handle.clone(),
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
