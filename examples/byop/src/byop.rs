/// BYOP (Build Your Own Preset) example for promkit.
///
/// This example demonstrates how to create a custom prompt using the `promkit` library.
/// It includes a text editor for input, a spinner for async task execution, and a task
/// monitor for managing background tasks.
/// The prompt allows users to enter text, start a heavy task (actually a simulated delay),
/// and see the results (actually show the input text) or errors
/// displayed in the UI. The example showcases the integration of various widgets and state
/// management techniques to create a responsive and interactive command-line application.
///
/// # Example Usage
/// To run this example, ensure you have the `promkit` library and its dependencies set up in
/// your Rust project. Then, execute the main function which initializes the BYOP prompt and
/// starts the event loop. You can interact with the prompt by typing commands and pressing
/// Enter to execute tasks. Use Ctrl+C to exit the prompt.
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
        spinner::{self, State},
        text_editor,
    },
    Prompt, Signal,
};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

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
    monitor_handle: JoinHandle<()>,
    task_handle: Arc<RwLock<Option<JoinHandle<Result<String>>>>>,
}

impl spinner::State for TaskMonitor {
    async fn is_idle(&self) -> bool {
        // Check if the task is currently running
        let running = self.task_handle.read().await;
        running.is_none() || running.as_ref().map_or(true, |handle| handle.is_finished())
    }
}

impl spinner::State for &TaskMonitor {
    async fn is_idle(&self) -> bool {
        // Delegate to the TaskMonitor
        (*self).is_idle().await
    }
}

impl TaskMonitor {
    fn new(renderer: SharedRenderer<Index>) -> Self {
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
        let task_handle = Arc::new(RwLock::new(None));
        let task_handle_internal = task_handle.clone();

        // Event handling daemon
        let monitor_handle = tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    TaskEvent::TaskStarted { handle } => {
                        // Store the handle for task management
                        {
                            let mut handle_guard = task_handle_internal.write().await;
                            *handle_guard = Some(handle);
                        }
                    }
                    TaskEvent::TaskCompleted { result } => {
                        // Clear the task handle
                        {
                            let mut handle_guard = task_handle_internal.write().await;
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
            monitor_handle,
            task_handle,
        }
    }

    fn abort_all(&self) {
        self.monitor_handle.abort();
        self.abort_task();
    }

    fn abort_task(&self) {
        if let Some(handle) = self.task_handle.blocking_read().as_ref() {
            handle.abort();
        }
    }
}

/// Build Your Own Prompt
struct BYOP {
    renderer: SharedRenderer<Index>,
    task_monitor: Arc<TaskMonitor>,
    readline: text_editor::State,
    spinner: Arc<spinner::Spinner>,
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

        // Abort any running tasks and clear the task monitor
        self.task_monitor.abort_all();

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

        Ok(Self {
            task_monitor: Arc::new(TaskMonitor::new(renderer.clone())),
            renderer,
            readline,
            spinner: Arc::new(
                spinner::Spinner::default()
                    .frames(spinner::frame::DOTS.clone())
                    .suffix("Executing...")
                    .duration(Duration::from_millis(100)),
            ),
        })
    }

    async fn start_heavy_task(&mut self) -> anyhow::Result<()> {
        // Check if task is already running
        if !self.task_monitor.is_idle().await {
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
                let is_running = !self.task_monitor.is_idle().await;
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
        let renderer = self.renderer.clone();
        let task_monitor = Arc::clone(&self.task_monitor);
        let spinner = Arc::clone(&self.spinner);

        let spinner_task = tokio::spawn(async move {
            spinner::run(
                spinner.as_ref(),
                task_monitor.as_ref(),
                Index::Spinner,
                renderer,
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
    BYOP::try_default().await?.spawn().await
}
